use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::consensus::serialize;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, Message, Secp256k1, SignOnly};
use bitcoin::util::bip143::SighashComponents;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::util::key;
use bitcoin::util::psbt::Map;
use bitcoin::{Address, Network, Script, SigHashType};
use firma::*;
use log::{debug, info};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Firma is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma")]
pub struct SignOptions {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// File containing the master key (xpriv...)
    #[structopt(short, long, parse(from_os_str))]
    key: PathBuf,

    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// derivations to consider if psbt doesn't contain HD paths
    #[structopt(short, long, default_value = "1000")]
    total_derivations: u32,

    /// PSBT json file
    psbt_file: PathBuf,
}

#[derive(Debug)]
struct PSBTSigner {
    pub psbt: PSBT,
    xprv: ExtendedPrivKey,
    secp: Secp256k1<SignOnly>,
    derivations: u32,
}

impl TryFrom<&SignOptions> for PSBTSigner {
    type Error = firma::Error;

    fn try_from(opt: &SignOptions) -> Result<Self> {
        let psbt_json = read_psbt(&opt.psbt_file)?;
        if psbt_json.signed_psbt.is_some() || psbt_json.only_sigs.is_some() {
            return err("The json psbt already contain signed_psbt or only_sigs, exiting to avoid risk of overwriting data");
        }
        let psbt = psbt_from_base64(&psbt_json.psbt)?;

        // TODO read key from .firma
        let xprv_string = std::fs::read_to_string(&opt.key)?;
        let xprv_json: PrivateMasterKeyJson = serde_json::from_str(&xprv_string)?;
        let xprv = ExtendedPrivKey::from_str(&xprv_json.xpriv)?;
        if xprv.network != opt.network {
            return err("Master key network is different from the network passed throug cli");
        }

        PSBTSigner::new(psbt, xprv, opt.total_derivations)
    }
}

impl PSBTSigner {
    pub fn new(psbt: PSBT, xprv: ExtendedPrivKey, derivations: u32) -> Result<Self> {
        let secp = Secp256k1::signing_only();
        Ok(PSBTSigner {
            psbt,
            xprv,
            secp,
            derivations,
        })
    }

    pub fn get_partial_sigs(&self) -> Result<Vec<u8>> {
        let mut only_partial_sigs = vec![];
        for input in self.psbt.inputs.iter() {
            for pair in input.get_pairs()?.iter() {
                if pair.key.type_value == 2u8 {
                    let vec = serialize(pair);
                    debug!("partial sig pair {}", hex::encode(&vec));
                    only_partial_sigs.extend(vec);
                }
            }
        }
        Ok(only_partial_sigs)
    }

    pub fn sign(&mut self) -> Result<bool> {
        self.init_hd_keypath_if_absent()?;

        for (i, input) in self.psbt.inputs.clone().iter().enumerate() {
            debug!("{} {:?}", i, input);
            match input.non_witness_utxo.clone() {
                Some(non_witness_utxo) => {
                    let prevout = self.psbt.global.unsigned_tx.input[i].previous_output;
                    assert_eq!(
                        non_witness_utxo.txid(),
                        prevout.txid,
                        "prevout doesn't match non_witness_utxo"
                    );
                    let script_pubkey = non_witness_utxo.output[prevout.vout as usize]
                        .clone()
                        .script_pubkey;
                    match input.redeem_script.clone() {
                        Some(redeem_script) => {
                            assert_eq!(
                                script_pubkey,
                                redeem_script.to_p2sh(),
                                "script_pubkey does not match the redeem script converted to p2sh"
                            );
                            self.sign_input(&redeem_script, i)?;
                        }
                        None => {
                            self.sign_input(&script_pubkey, i)?;
                        }
                    };
                }
                None => {
                    let witness_utxo = input
                        .clone()
                        .witness_utxo
                        .expect("both witness_utxo and non_witness_utxo are none");
                    let script = match input.clone().redeem_script {
                        Some(script) => {
                            assert_eq!(witness_utxo.script_pubkey, script.to_p2sh(), "witness_utxo script_pubkey doesn't match the redeem script converted to p2sh");
                            script
                        }
                        None => witness_utxo.script_pubkey,
                    };
                    if script.is_v0_p2wpkh() {
                        let script = to_p2pkh(&script.as_bytes()[2..]);
                        assert!(script.is_p2pkh(), "it is not a p2pkh script");
                        self.sign_input(&script, i)?;
                    } else {
                        let wit_script = input
                            .clone()
                            .witness_script
                            .expect("witness_script is none");
                        assert_eq!(
                            script,
                            wit_script.to_v0_p2wsh(),
                            "script and witness script to v0 p2wsh doesn't match"
                        );
                        self.sign_input(&wit_script, i)?;
                    }
                }
            }
        }
        Ok(true)
    }

    fn init_hd_keypath_if_absent(&mut self) -> Result<()> {
        // temp code for handling psbt generated from core without hd paths
        let outputs_empty = self.psbt.inputs.iter().any(|i| i.hd_keypaths.is_empty());
        let inputs_empty = self.psbt.outputs.iter().any(|o| o.hd_keypaths.is_empty());

        if outputs_empty || inputs_empty {
            info!("Provided PSBT does not contain all HD key paths, trying to deduce them...");
            let mut keys = HashMap::new();
            for i in 0..=1 {
                let derivation_path = DerivationPath::from_str(&format!("m/{}", i))?;
                let first = self.xprv.derive_priv(&self.secp, &derivation_path)?;
                for j in 0..=self.derivations {
                    let derivation_path = DerivationPath::from_str(&format!("m/{}", j))?;
                    let derived = first.derive_priv(&self.secp, &derivation_path)?;
                    let derived_pubkey = ExtendedPubKey::from_private(&self.secp, &derived);
                    let complete_derivation_path =
                        DerivationPath::from_str(&format!("m/{}/{}", i, j))?;
                    keys.insert(
                        derived_pubkey.public_key,
                        (self.xprv.fingerprint(&self.secp), complete_derivation_path),
                    );
                }
            }

            for input in self.psbt.inputs.iter_mut() {
                if let Some(ref witness_script) = input.witness_script {
                    let script_keys = extract_pub_keys(&witness_script)?;
                    for key in script_keys {
                        if keys.contains_key(&key) {
                            input.hd_keypaths.insert(
                                key.clone(),
                                keys.get(&key).ok_or_else(fn_err("key not found"))?.clone(),
                            );
                        }
                    }
                }
            }

            for output in self.psbt.outputs.iter_mut() {
                if let Some(ref witness_script) = output.witness_script {
                    let script_keys = extract_pub_keys(&witness_script)?;
                    for key in script_keys {
                        if keys.contains_key(&key) {
                            output.hd_keypaths.insert(
                                key.clone(),
                                keys.get(&key).ok_or_else(fn_err("key not found"))?.clone(),
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn sign_input(&mut self, script: &Script, input_index: usize) -> Result<()> {
        let input = &mut self.psbt.inputs[input_index];
        let tx = &self.psbt.global.unsigned_tx;
        let is_witness = input.non_witness_utxo.is_none();
        let my_fing = self.xprv.fingerprint(&self.secp);

        for (pubkey, (fing, child)) in input.hd_keypaths.iter() {
            if fing != &my_fing {
                continue;
            }
            let privkey = self.xprv.derive_priv(&self.secp, &child)?;
            let derived_pubkey =
                secp256k1::PublicKey::from_secret_key(&self.secp, &privkey.private_key.key);
            if pubkey.key != derived_pubkey {
                return err("pubkey derived and expected differs even if fingerprint matches!");
            }
            let (hash, sighash);
            if is_witness {
                let wutxo = input.clone().witness_utxo;
                let value = wutxo.ok_or_else(fn_err("witness_utxo is empty"))?.value;
                let cmp = SighashComponents::new(tx);
                hash = cmp.sighash_all(&tx.input[input_index], script, value);
                sighash = input.sighash_type.unwrap_or(SigHashType::All);
            } else {
                sighash = input.sighash_type.ok_or_else(fn_err("sighash empty"))?;
                hash = tx.signature_hash(input_index, &script, sighash.as_u32());
            };
            let msg = &Message::from_slice(&hash.into_inner()[..])?;
            let key = &privkey.private_key.key;
            let signature = self.secp.sign(msg, key);
            let mut signature = signature.serialize_der().to_vec();
            signature.push(sighash.as_u32() as u8); // TODO how to properly do this?
            input.partial_sigs.insert(pubkey.clone(), signature);
        }
        Ok(())
    }

    pub fn pretty_print(&self) -> Result<()> {
        let mut input_values: Vec<u64> = vec![];
        let mut output_values: Vec<u64> = vec![];
        let tx = &self.psbt.global.unsigned_tx;
        let network = self.xprv.network;
        let ins = tx.input.iter();
        let vouts: Vec<usize> = ins.map(|el| el.previous_output.vout as usize).collect();
        for (i, input) in self.psbt.inputs.iter().enumerate() {
            let val = match (&input.non_witness_utxo, &input.witness_utxo) {
                (Some(val), None) => {
                    let vout = *vouts.get(i).ok_or_else(fn_err("can't find vout"))?;
                    val.output
                        .get(vout)
                        .ok_or_else(fn_err("can't find value"))?
                        .value
                }
                (None, Some(val)) => val.value,
                _ => return err("witness_utxo and non_witness_utxo are both None or both Some"),
            };
            input_values.push(val);
        }

        info!("\ninputs [# prevout:vout value]:");
        for (i, input) in tx.input.iter().enumerate() {
            info!(
                "#{} {}:{} ({}) {}",
                i,
                input.previous_output.txid,
                input.previous_output.vout,
                derivation_paths(&self.psbt.inputs[i].hd_keypaths),
                input_values[i],
            );
        }
        info!("\noutputs [# script address amount]:");
        for (i, output) in tx.output.iter().enumerate() {
            // TODO calculate if it is mine
            info!(
                "#{} {} {} ({}) {}",
                i,
                hex::encode(&output.script_pubkey.as_bytes()),
                Address::from_script(&output.script_pubkey, network)
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown address".into()),
                derivation_paths(&self.psbt.outputs[i].hd_keypaths),
                output.value
            );
            output_values.push(output.value);
        }
        // TODO show privacy analysis like blockstream.info
        let fee = input_values.iter().sum::<u64>() - output_values.iter().sum::<u64>();

        let tx_vbytes = tx.get_weight() / 4;
        let estimated_tx_vbytes = estimate_weight(&self.psbt)? / 4;
        let estimated_fee_rate = fee as f64 / estimated_tx_vbytes as f64;

        info!("");
        info!("absolute fee       : {:>6}   satoshi", fee);
        info!("unsigned tx        : {:>6}   vbyte", tx_vbytes);
        info!("estimated tx       : {:>6}   vbyte", estimated_tx_vbytes);
        info!("estimated fee rate : {:>8.1} sat/vbyte", estimated_fee_rate);
        Ok(())
    }

    fn update_psbt_file(&self, psbt_file: &PathBuf) -> Result<()> {
        let mut psbt_json = read_psbt(&psbt_file)?;
        let partial_sigs = self.get_partial_sigs()?;
        psbt_json.only_sigs = Some(base64::encode(&partial_sigs));
        let signed_psbt = base64::encode(&serialize(&self.psbt));
        psbt_json.signed_psbt = Some(signed_psbt);
        std::fs::write(&psbt_file, serde_json::to_string_pretty(&psbt_json)?)?;
        Ok(())
    }
}

pub fn start(opt: &SignOptions) -> Result<()> {
    let mut psbt_signer = PSBTSigner::try_from(opt)?;
    debug!("{:#?}", psbt_signer);

    let signed = psbt_signer.sign()?;
    psbt_signer.pretty_print()?;

    if signed {
        psbt_signer.update_psbt_file(&opt.psbt_file)?;
        info!("\nAdded signatures, wrote {:?}", &opt.psbt_file);
    } else {
        info!("\nNo signature added");
    }

    Ok(())
}

fn to_p2pkh(pubkey_hash: &[u8]) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(pubkey_hash)
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}

pub fn derivation_paths(
    hd_keypaths: &BTreeMap<key::PublicKey, (Fingerprint, DerivationPath)>,
) -> String {
    let mut vec = vec![];
    for (_, (_, path)) in hd_keypaths.iter() {
        vec.push(format!("{:?}", path));
    }
    vec.sort();
    vec.dedup();
    vec.join(", ")
}

#[cfg(test)]
mod tests {
    use crate::sign::*;

    fn test_sign(psbt_to_sign: &mut PSBT, psbt_signed: &PSBT, xprv: &str) -> Result<()> {
        let xprv = std::str::FromStr::from_str(xprv)?;
        let mut psbt_signer = PSBTSigner::new(psbt_to_sign.clone(), xprv, 10)?;
        psbt_signer.sign()?;
        assert_eq!(&psbt_signer.psbt, psbt_signed);
        Ok(())
    }

    fn extract_psbt(bytes: &[u8]) -> Result<(PSBT, PSBT, String)> {
        let expect: firma::PsbtJson = serde_json::from_slice(bytes)?;
        let psbt_to_sign = psbt_from_base64(&expect.psbt)?;
        let psbt_str = expect
            .signed_psbt
            .ok_or_else(fn_err("signed_psbt is empty"))?;
        let psbt_signed = psbt_from_base64(psbt_str.as_ref())?;
        Ok((psbt_to_sign, psbt_signed, psbt_str.clone()))
    }

    fn perc_diff_with_core(psbt: &PSBT, core: usize) -> Result<bool> {
        let esteem = (estimate_weight(psbt)? / 4) as f64;
        let core = core as f64;
        let perc = ((esteem - core) / esteem).abs();
        Ok(perc < 0.1) // TODO reduce this 10% by improving estimation of the bip tx
    }

    #[test]
    fn test_psbt() -> Result<()> {
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.signed.json");
        let (mut psbt_to_sign, psbt_signed, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.key");
        let key: firma::PrivateMasterKeyJson = serde_json::from_slice(bytes)?;
        test_sign(&mut psbt_to_sign, &psbt_signed, &key.xpriv)?;
        assert!(perc_diff_with_core(&psbt_to_sign, 462)?); // 462 is estimated_vsize from analyzepsbt

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.signed.json");
        let (mut psbt_to_sign, mut psbt1, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.key");
        let key: firma::PrivateMasterKeyJson = serde_json::from_slice(bytes)?;
        test_sign(&mut psbt_to_sign, &psbt1, &key.xpriv)?;
        assert!(perc_diff_with_core(&psbt_to_sign, 192)?);

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.signed.json");
        let (mut psbt_to_sign, psbt2, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.key");
        let key: firma::PrivateMasterKeyJson = serde_json::from_slice(bytes)?;
        test_sign(&mut psbt_to_sign, &psbt2, &key.xpriv)?;

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.signed.json");
        let (_, psbt_complete, psbt_signed_complete) = extract_psbt(bytes)?;

        psbt1.merge(psbt2)?;

        assert_eq!(psbt1, psbt_complete);
        assert_eq!(psbt_to_base64(&psbt1), psbt_signed_complete);

        Ok(())
    }

    pub fn psbt_to_base64(psbt: &PSBT) -> String {
        base64::encode(&serialize(psbt))
    }
}
