use crate::print::pretty_print;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::consensus::serialize;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, Message, Secp256k1, SignOnly};
use bitcoin::util::bip143::SighashComponents;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::util::psbt::Map;
use bitcoin::{Network, Script, SigHashType};
use firma::common::{extract_pub_keys, read_psbt, read_psbt_json};
use firma::*;
use log::{debug, info};
use serde_json::{to_value, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Sign a Partially Signed Bitcoin Transaction (PSBT) with a key.
#[derive(StructOpt, Debug)]
pub struct SignOptions {
    /// File containing the master key (xpriv...)
    #[structopt(short, long, parse(from_os_str))]
    key: PathBuf,

    /// derivations to consider if psbt doesn't contain HD paths
    #[structopt(short, long, default_value = "1000")]
    total_derivations: u32,

    /// File containing the wallet descriptor, show if outputs are mine.
    #[structopt(short, long, parse(from_os_str))]
    wallet_descriptor_file: PathBuf,

    /// PSBT json file
    psbt_file: PathBuf,
}

pub struct SignResult {
    signed: bool,
    added_paths: bool,
}

#[derive(Debug)]
struct PSBTSigner {
    pub psbt: PSBT,
    xprv: ExtendedPrivKey,
    secp: Secp256k1<SignOnly>,
    network: Network, // even if network is included in xprv, regtest is equal to testnet there, so we need this
    derivations: u32,
}

impl PSBTSigner {
    fn from_opt(opt: &SignOptions, network: Network) -> Result<Self> {
        let psbt = read_psbt(&opt.psbt_file, false)?;

        // TODO read key from .firma
        let xprv_json = read_key(&opt.key)?;
        let xprv = xprv_json.xprv.clone();
        if !(network == xprv.network
            || (network == Network::Regtest && xprv.network == Network::Testnet))
        {
            return err("Master key network is different from the network passed through cli");
        }

        PSBTSigner::new(psbt, xprv, opt.total_derivations, network)
    }

    pub fn new(
        psbt: PSBT,
        xprv: ExtendedPrivKey,
        derivations: u32,
        network: Network,
    ) -> Result<Self> {
        let secp = Secp256k1::signing_only();
        Ok(PSBTSigner {
            psbt,
            xprv,
            secp,
            derivations,
            network,
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

    pub fn sign(&mut self) -> Result<SignResult> {
        let initial_inputs = self.psbt.inputs.clone();
        let added_paths = self.init_hd_keypath_if_absent()?;

        for (i, input) in self.psbt.inputs.clone().iter().enumerate() {
            debug!("{} {:?}", i, input);
            match input.non_witness_utxo.clone() {
                Some(non_witness_utxo) => {
                    let prevout = self.psbt.global.unsigned_tx.input[i].previous_output;
                    if non_witness_utxo.txid() != prevout.txid {
                        return err("prevout doesn't match non_witness_utxo");
                    }
                    let script_pubkey = non_witness_utxo.output[prevout.vout as usize]
                        .clone()
                        .script_pubkey;
                    match input.redeem_script.clone() {
                        Some(redeem_script) => {
                            if script_pubkey != redeem_script.to_p2sh() {
                                return err("script_pubkey does not match the redeem script converted to p2sh");
                            }
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
                            if witness_utxo.script_pubkey != script.to_p2sh() {
                                return err("witness_utxo script_pubkey doesn't match the redeem script converted to p2sh");
                            }
                            script
                        }
                        None => witness_utxo.script_pubkey,
                    };
                    if script.is_v0_p2wpkh() {
                        let script = to_p2pkh(&script.as_bytes()[2..]);
                        if !script.is_p2pkh() {
                            return err("it is not a p2pkh script");
                        }
                        self.sign_input(&script, i)?;
                    } else {
                        let wit_script = input
                            .clone()
                            .witness_script
                            .expect("witness_script is none");
                        if script != wit_script.to_v0_p2wsh() {
                            return err("script and witness script to v0 p2wsh doesn't match");
                        }
                        self.sign_input(&wit_script, i)?;
                    }
                }
            }
        }
        let signed = self.psbt.inputs != initial_inputs;
        Ok(SignResult {
            added_paths,
            signed,
        })
    }

    fn init_hd_keypath_if_absent(&mut self) -> Result<bool> {
        // temp code for handling psbt generated from core without hd paths
        let outputs_empty = self.psbt.inputs.iter().any(|i| i.hd_keypaths.is_empty());
        let inputs_empty = self.psbt.outputs.iter().any(|o| o.hd_keypaths.is_empty());

        let mut added = false;
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
                            added = true;
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
                            added = true;
                        }
                    }
                }
            }
        }
        if added {
            info!("Added HD key paths\n");
        }
        Ok(added)
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

    fn update_psbt_file(&self, psbt_file: &PathBuf) -> Result<()> {
        let mut psbt_json = read_psbt_json(&psbt_file)?;
        let partial_sigs = self.get_partial_sigs()?;
        psbt_json.only_sigs = Some(base64::encode(&partial_sigs));
        let signed_psbt = base64::encode(&serialize(&self.psbt));
        psbt_json.signed_psbt = Some(signed_psbt);
        std::fs::write(&psbt_file, serde_json::to_string_pretty(&psbt_json)?)?;
        Ok(())
    }

    fn pretty_print(&self, fingerprints: &HashSet<Fingerprint>) -> Result<PsbtPrettyPrint> {
        pretty_print(&self.psbt, self.network, fingerprints)
    }
}

pub fn start(opt: &SignOptions, network: Network) -> Result<Value> {
    let wallet = read_wallet(&opt.wallet_descriptor_file)?;
    let mut psbt_signer = PSBTSigner::from_opt(opt, network)?;
    debug!("{:?}", psbt_signer);

    let sign_result = psbt_signer.sign()?;
    let mut psbt_print = psbt_signer.pretty_print(&wallet.fingerprints)?;

    if sign_result.added_paths {
        psbt_print.info.push("Added paths".to_string());
    }
    if sign_result.signed {
        psbt_signer.update_psbt_file(&opt.psbt_file)?;
        psbt_print.info.push("Added signatures".to_string());
        psbt_print.psbt_file = opt.psbt_file.clone();
    } else {
        psbt_print.info.push("No signature added".to_string());
    }

    Ok(to_value(psbt_print)?)
}

pub fn read_key(path: &PathBuf) -> Result<PrivateMasterKey> {
    let xprv_string = std::fs::read(path)?;
    Ok(serde_json::from_slice(&xprv_string)?)
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

#[cfg(test)]
mod tests {
    use crate::sign::*;
    use firma::common::{psbt_from_base64, estimate_weight};
    use bitcoin::util::bip32::ExtendedPubKey;

    fn test_sign(
        psbt_to_sign: &mut PSBT,
        psbt_signed: &PSBT,
        xprv: &ExtendedPrivKey,
    ) -> Result<()> {
        let mut psbt_signer =
            PSBTSigner::new(psbt_to_sign.clone(), xprv.clone(), 10, xprv.network)?;
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
        let secp = Secp256k1::signing_only();
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.signed.json");
        let (mut psbt_to_sign, psbt_signed, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.key");
        let key: firma::PrivateMasterKey = serde_json::from_slice(bytes)?;
        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );
        test_sign(&mut psbt_to_sign, &psbt_signed, &key.xprv)?;
        assert!(perc_diff_with_core(&psbt_to_sign, 462)?); // 462 is estimated_vsize from analyzepsbt

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.signed.json");
        let (mut psbt_to_sign, mut psbt1, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.key");
        let key: firma::PrivateMasterKey = serde_json::from_slice(bytes)?;
        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );
        test_sign(&mut psbt_to_sign, &psbt1, &key.xprv)?;
        assert!(perc_diff_with_core(&psbt_to_sign, 192)?);

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.signed.json");
        let (mut psbt_to_sign, psbt2, _) = extract_psbt(bytes)?;
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.key");
        let key: firma::PrivateMasterKey = serde_json::from_slice(bytes)?;
        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );
        test_sign(&mut psbt_to_sign, &psbt2, &key.xprv)?;

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
