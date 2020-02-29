use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::{Builder, Instruction::PushBytes};
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, Message, Secp256k1, SignOnly};
use bitcoin::util::bip143::SighashComponents;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::util::key;
use bitcoin::util::psbt::Map;
use bitcoin::util::psbt::{Input, PartiallySignedTransaction};
use bitcoin::{Address, Network, Script, SigHashType, Transaction};
use firma::{MasterKeyJson, PsbtJson};
use log::{debug, info};
use log::{Level, Metadata, Record};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

type PSBT = PartiallySignedTransaction;

/// Firma is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma")]
pub struct SignOptions {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Decode a PSBT and print informations
    #[structopt(short, long)]
    decode: bool,

    /// File containing the master key (xpriv...)
    #[structopt(short, long, parse(from_os_str))]
    key: Option<PathBuf>,

    /// Name of the wallet
    #[structopt(short, long)]
    wallet_name: String,

    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// derivations to consider if psbt doesn't contain HD paths
    #[structopt(short, long, default_value = "1000")]
    total_derivations: u32,

    /// PSBT json file
    file: PathBuf,
}

pub fn start(opt: &SignOptions) -> Result<(), Box<dyn Error>> {
    debug!("{:#?}", opt);
    let json = fs::read_to_string(&opt.file).unwrap();
    let mut json: PsbtJson = serde_json::from_str(&json).unwrap();
    debug!("{:#?}", json);

    let mut psbt = psbt_from_base64(&json.psbt)?;
    debug!("{:#?}", psbt);

    let initial_partial_sigs = get_partial_sigs(&psbt);

    if opt.decode {
        pretty_print(&psbt, opt.network)
    } else {
        if json.signed_psbt.is_some() || json.only_sigs.is_some() {
            info!("The json psbt already contain signed_psbt or only_sigs, exiting to avoid risk of overwriting data");
            return Ok(());
        }

        start_psbt(&opt, &mut psbt, &mut json)?;

        let partial_sigs = get_partial_sigs(&psbt);

        if !partial_sigs.is_empty() {
            json.only_sigs = Some(base64::encode(&partial_sigs));
        }

        if initial_partial_sigs != partial_sigs {
            fs::write(&opt.file, serde_json::to_string_pretty(&json).unwrap())
                .unwrap_or_else(|_| panic!("Unable to write {:?}", &opt.file));
            info!("\nAdded signatures, wrote {:?}", &opt.file);
        } else {
            info!("\nNo signature added");
        }
    }

    Ok(())
}

fn get_partial_sigs(psbt: &PSBT) -> Vec<u8> {
    let mut only_partial_sigs = vec![];
    for input in psbt.inputs.iter() {
        for pair in input.get_pairs().unwrap().iter() {
            if pair.key.type_value == 2u8 {
                let vec = serialize(pair);
                debug!("partial sig pair {}", hex::encode(&vec));
                only_partial_sigs.extend(vec);
            }
        }
    }
    only_partial_sigs
}

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() <= Level::Warn {
                println!("{} - {}", record.level(), record.args());
            } else {
                println!("{}", record.args());
            }
        }
    }

    fn flush(&self) {}
}

pub fn start_psbt(
    opt: &SignOptions,
    psbt: &mut PSBT,
    json: &mut PsbtJson,
) -> Result<(), Box<dyn Error>> {
    if !opt.decode && opt.key.is_none() {
        info!("--key <file> or --decode must be used");
        std::process::exit(-1);
    }

    let xpriv = fs::read_to_string(opt.key.as_ref().unwrap())
        .unwrap_or_else(|_| panic!("Unable to read file {:?}", &opt.key));

    let xpriv: MasterKeyJson = serde_json::from_str(&xpriv).unwrap();
    let xpriv = ExtendedPrivKey::from_str(&xpriv.xpriv)?;
    assert_eq!(xpriv.network, opt.network);
    sign_psbt(psbt, &xpriv, Some(opt.total_derivations));
    pretty_print(psbt, xpriv.network);

    let signed_psbt = base64::encode(&serialize(psbt));
    if signed_psbt != json.psbt {
        json.signed_psbt = Some(signed_psbt);
    }

    Ok(())
}

fn extract_pub_keys(script: &Script) -> Vec<key::PublicKey> {
    let mut result = vec![];
    for instruct in script.iter(false) {
        if let PushBytes(a) = instruct {
            if a.len() == 33 {
                result.push(key::PublicKey::from_slice(&a).unwrap());
            }
        }
    }
    result
}

fn sign(
    tx: &Transaction,
    script: &Script,
    input_index: usize,
    input: &mut Input,
    xpriv: &ExtendedPrivKey,
    secp: &Secp256k1<SignOnly>,
) {
    let is_witness = input.non_witness_utxo.is_none();
    let my_fing = xpriv.fingerprint(secp);

    for (pubkey, (fing, child)) in input.hd_keypaths.iter() {
        if fing == &my_fing {
            let privkey = xpriv.derive_priv(&secp, &child).unwrap();
            let derived_pubkey =
                secp256k1::PublicKey::from_secret_key(&secp, &privkey.private_key.key);
            assert_eq!(pubkey.key, derived_pubkey);

            let (hash, sighash) = if is_witness {
                (
                    SighashComponents::new(tx).sighash_all(
                        &tx.input[input_index],
                        script,
                        input.clone().witness_utxo.unwrap().value,
                    ),
                    input.sighash_type.unwrap_or(SigHashType::All),
                ) // TODO how to handle other sighash type?
            } else {
                let sighash = input.sighash_type.unwrap();
                (
                    tx.signature_hash(input_index, &script, sighash.as_u32()),
                    sighash,
                )
            };
            let signature = secp.sign(
                &Message::from_slice(&hash.into_inner()[..]).unwrap(),
                &privkey.private_key.key,
            );
            let mut signature = signature.serialize_der().to_vec();
            signature.push(sighash.as_u32() as u8); // TODO how to properly do this?
            input.partial_sigs.insert(pubkey.clone(), signature);
        }
    }
}

fn sign_psbt(psbt: &mut PSBT, xpriv: &ExtendedPrivKey, derivations: Option<u32>) {
    let secp = &Secp256k1::signing_only();

    init_hd_keypath_if_absent(psbt, xpriv, derivations, secp);

    let tx = &psbt.global.unsigned_tx;

    for (i, mut input) in psbt.inputs.iter_mut().enumerate() {
        debug!("{} {:?}", i, input);
        match input.non_witness_utxo.clone() {
            Some(non_witness_utxo) => {
                let prevout = tx.input[i].previous_output;
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
                        sign(tx, &redeem_script, i, &mut input, xpriv, secp);
                    }
                    None => {
                        sign(tx, &script_pubkey, i, &mut input, xpriv, secp);
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
                    sign(tx, &script, i, &mut input, xpriv, secp);
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
                    sign(tx, &wit_script, i, &mut input, xpriv, secp);
                }
            }
        }
    }
}

fn init_hd_keypath_if_absent(
    psbt: &mut PartiallySignedTransaction,
    xpriv: &ExtendedPrivKey,
    derivations: Option<u32>,
    secp: &Secp256k1<SignOnly>,
) {
    // temp code for handling psbt generated from core without hd paths
    let mut have_hd_keypaths = true;
    for input in psbt.inputs.iter() {
        if input.hd_keypaths.is_empty() {
            have_hd_keypaths = false;
        }
    }
    for output in psbt.outputs.iter() {
        if output.hd_keypaths.is_empty() {
            have_hd_keypaths = false;
        }
    }
    if !have_hd_keypaths {
        info!("Provided PSBT does not contain HD key paths, trying to deduce them...");
        let mut keys = HashMap::new();
        for i in 0..=1 {
            let derivation_path = DerivationPath::from_str(&format!("m/{}", i)).unwrap();
            let first = xpriv.derive_priv(&secp, &derivation_path).unwrap();
            for j in 0..=derivations.unwrap_or(1000) {
                let derivation_path = DerivationPath::from_str(&format!("m/{}", j)).unwrap();
                let derived = first.derive_priv(&secp, &derivation_path).unwrap();
                let derived_pubkey = ExtendedPubKey::from_private(&secp, &derived);
                let complete_derivation_path =
                    DerivationPath::from_str(&format!("m/{}/{}", i, j)).unwrap();
                keys.insert(
                    derived_pubkey.public_key,
                    (xpriv.fingerprint(&secp), complete_derivation_path),
                );
            }
        }

        for input in psbt.inputs.iter_mut() {
            if input.witness_script.is_some() {
                let script_keys = extract_pub_keys(input.witness_script.as_ref().unwrap());
                for key in script_keys {
                    if keys.contains_key(&key) {
                        input
                            .hd_keypaths
                            .insert(key.clone(), keys.get(&key).unwrap().clone());
                    }
                }
            }
        }

        for output in psbt.outputs.iter_mut() {
            if output.witness_script.is_some() {
                let script_keys = extract_pub_keys(output.witness_script.as_ref().unwrap());
                for key in script_keys {
                    if keys.contains_key(&key) {
                        output
                            .hd_keypaths
                            .insert(key.clone(), keys.get(&key).unwrap().clone());
                    }
                }
            }
        }
    }
}

fn estimate_weight(psbt: &PSBT) -> usize {
    let unsigned_weight = psbt.global.unsigned_tx.get_weight();
    let mut spending_weight = 0usize;

    for input in psbt.inputs.iter() {
        let (script, factor) = match (&input.redeem_script, &input.witness_script) {
            (Some(redeem_script), None) => (redeem_script, 4),
            (_, Some(witness_script)) => (witness_script, 1), // factor=1 for segwit discount
            _ => panic!("both redeem and witness script are None"),
        };
        //TODO signature are less in NofM where N<M
        let current = script.len() + expected_signatures(script) * 72; // using 72 as average signature size
        spending_weight += current * factor;
    }

    unsigned_weight + spending_weight
}

fn expected_signatures(script: &Script) -> usize {
    let bytes = script.as_bytes();
    if bytes.len() > 1 && bytes.last().unwrap() == &opcodes::all::OP_CHECKMULTISIG.into_u8() {
        read_pushnum(bytes[0])
            .map(|el| el as usize)
            .unwrap_or(0usize)
    } else {
        extract_pub_keys(script).len()
    }
}

fn read_pushnum(value: u8) -> Option<u8> {
    if value >= opcodes::all::OP_PUSHNUM_1.into_u8()
        && value <= opcodes::all::OP_PUSHNUM_16.into_u8()
    {
        Some(value - opcodes::all::OP_PUSHNUM_1.into_u8() + 1)
    } else {
        None
    }
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

pub fn psbt_from_base64(s: &str) -> Result<PSBT, Box<dyn Error>> {
    let bytes = base64::decode(s)?;
    let psbt = deserialize(&bytes)?;
    Ok(psbt)
}

pub fn psbt_to_base64(psbt: &PSBT) -> String {
    base64::encode(&serialize(psbt))
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

pub fn pretty_print(psbt: &PSBT, network: Network) {
    let mut input_values: Vec<u64> = vec![];
    let mut output_values: Vec<u64> = vec![];

    info!("");

    let vouts: Vec<usize> = psbt
        .global
        .unsigned_tx
        .input
        .iter()
        .map(|el| el.previous_output.vout as usize)
        .collect();
    for (i, input) in psbt.inputs.iter().enumerate() {
        let val = match (&input.non_witness_utxo, &input.witness_utxo) {
            (Some(val), None) => val.output.get(*vouts.get(i).unwrap()).unwrap().value,
            (None, Some(val)) => val.value,
            _ => panic!("witness_utxo and non_witness_utxo are both None or both Some"),
        };
        input_values.push(val);
    }

    let transaction = &psbt.global.unsigned_tx;
    info!("\ninputs [# prevout:vout value]:");
    for (i, input) in transaction.input.iter().enumerate() {
        info!(
            "#{} {}:{} ({}) {}",
            i,
            input.previous_output.txid,
            input.previous_output.vout,
            derivation_paths(&psbt.inputs[i].hd_keypaths),
            input_values[i],
        );
    }
    info!("\noutputs [# script address amount]:");
    for (i, output) in transaction.output.iter().enumerate() {
        // TODO calculate if it is mine
        info!(
            "#{} {} {} ({}) {}",
            i,
            hex::encode(&output.script_pubkey.as_bytes()),
            Address::from_script(&output.script_pubkey, network)
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown address".into()),
            derivation_paths(&psbt.outputs[i].hd_keypaths),
            output.value
        );
        output_values.push(output.value);
    }
    // TODO show privacy analysis like blockstream.info
    let fee = input_values.iter().sum::<u64>() - output_values.iter().sum::<u64>();

    let tx_vbytes = psbt.global.unsigned_tx.get_weight() / 4;
    let estimated_tx_vbytes = estimate_weight(&psbt) / 4;
    let estimated_fee_rate = fee as f64 / estimated_tx_vbytes as f64;

    info!("");
    info!("absolute fee       : {:>6} satoshi", fee);
    info!("unsigned tx        : {:>6} vbyte", tx_vbytes);
    info!("estimated tx       : {:>6} vbyte", estimated_tx_vbytes);
    info!("estimated fee rate : {:>6.0} sat/vbyte", estimated_fee_rate);
}

#[cfg(test)]
mod tests {
    use crate::sign::*;
    use bitcoin::util::bip32::ExtendedPrivKey;
    use firma::{MasterKeyJson, PsbtJson};
    use std::str::FromStr;

    fn test_sign(psbt_to_sign: &mut PSBT, psbt_signed: &PSBT, xpriv: &str) {
        let xpriv = ExtendedPrivKey::from_str(xpriv).unwrap();
        sign_psbt(psbt_to_sign, &xpriv, Some(10u32));
        assert_eq!(psbt_to_sign, psbt_signed);
    }

    fn extract_psbt(bytes: &[u8]) -> (PSBT, PSBT, String) {
        let expected: PsbtJson = serde_json::from_slice(bytes).unwrap();
        let psbt_to_sign = psbt_from_base64(&expected.psbt).unwrap();
        let psbt_signed = psbt_from_base64(expected.signed_psbt.as_ref().unwrap()).unwrap();
        (
            psbt_to_sign,
            psbt_signed,
            expected.signed_psbt.unwrap().clone(),
        )
    }

    fn perc_diff_with_core(psbt: &PSBT, core: usize) -> bool {
        let esteem = (estimate_weight(psbt) / 4) as f64;
        let core = core as f64;
        let perc = ((esteem - core) / esteem).abs();
        perc < 0.1 // TODO reduce this 10% by improving estimation of the bip tx
    }

    #[test]
    fn test_psbt() {
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.signed.json");
        let (mut psbt_to_sign, psbt_signed, _) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.key");
        let key: MasterKeyJson = serde_json::from_slice(bytes).unwrap();
        test_sign(&mut psbt_to_sign, &psbt_signed, &key.xpriv);
        assert!(perc_diff_with_core(&psbt_to_sign, 462)); // 462 is estimated_vsize from analyzepsbt

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.signed.json");
        let (mut psbt_to_sign, mut psbt1, _) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.key");
        let key: MasterKeyJson = serde_json::from_slice(bytes).unwrap();
        test_sign(&mut psbt_to_sign, &psbt1, &key.xpriv);
        assert!(perc_diff_with_core(&psbt_to_sign, 192));

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.signed.json");
        let (mut psbt_to_sign, psbt2, _) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.key");
        let key: MasterKeyJson = serde_json::from_slice(bytes).unwrap();
        test_sign(&mut psbt_to_sign, &psbt2, &key.xpriv);

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.signed.json");
        let (_, psbt_complete, psbt_signed_complete) = extract_psbt(bytes);

        psbt1.merge(psbt2).unwrap();

        assert_eq!(psbt1, psbt_complete);
        assert_eq!(psbt_to_base64(&psbt1), psbt_signed_complete)
    }

    #[test]
    fn test_miniscript() {

        //let desc = miniscript::Descriptor::<bitcoin::PublicKey>::from_str("sh(wsh(or_d(c:pk(020e0338c96a8870479f2396c373cc7696ba124e8635d41b0ea581112b67817261), c:pk(020e0338c96a8870479f2396c373cc7696ba124e8635d41b0ea581112b67817261))))").unwrap();

        // Derive the P2SH address
        /*assert_eq!(
            desc.address(bitcoin::Network::Bitcoin).unwrap().to_string(),
            "32aAVauGwencZwisuvd3anhhhQhNZQPyHv"
        );*/
        // TODO wait integration of descriptor with master keys
    }
}
