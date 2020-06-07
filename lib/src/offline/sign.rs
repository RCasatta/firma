use crate::offline::print::pretty_print;
use crate::qr::save_qrs;
use crate::*;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, Message, Secp256k1, SignOnly};
use bitcoin::util::bip143::SighashComponents;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::util::psbt::{raw, Map};
use bitcoin::{Network, Script, SigHashType, Txid};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Sign a Partially Signed Bitcoin Transaction (PSBT) with a key.
#[derive(StructOpt, Debug, Serialize, Deserialize)]
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
    //TODO remove and read all the available wallets?
    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,

    /// PSBT json file
    psbt_file: PathBuf,

    /// Allow any derivations (to avoid ramson attacks, by default only 2 levels are allowed, and the first level must be 0 or 1)
    #[structopt(long)]
    allow_any_derivations: bool,
}

pub struct SignResult {
    signed: bool,
    added_paths: bool,
}

#[derive(Debug)]
struct PSBTSigner {
    pub psbt: PSBT,
    psbts_dir: PathBuf,
    xprv: ExtendedPrivKey,
    secp: Secp256k1<SignOnly>,
    network: Network, // even if network is included in xprv, regtest is equal to testnet there, so we need this
    derivations: u32,
    allow_any_derivations: bool,
}

/// extract field name in the PSBT extra field if present
pub fn get_psbt_name(psbt: &PSBT) -> Option<String> {
    psbt.global.unknown.get(&get_name_key()).map(|v| {
        std::str::from_utf8(v)
            .expect("PSBT name not utf8")
            .to_string()
    }) // TODO remove expect
}

pub fn save_psbt_opt(datadir: &str, network: Network, opt: &SavePSBTOptions) -> Result<()> {
    info!("save_psbt_opt {:?}", opt);
    let bytes = opt.psbt.as_bytes()?;
    let mut psbt: PSBT = deserialize(&bytes)?;
    let mut psbts_dir: PathBuf = datadir.into();
    psbts_dir.push(format!("{}", network));
    psbts_dir.push("psbts");
    save_psbt(&mut psbt, &mut psbts_dir, opt.qr_version)?;
    Ok(())
}

/// Search existing psbt, if one matches the txid, return that name, otherwise it gives a new unused name
fn get_name(psbts_dir: &PathBuf, txid: &Txid) -> Result<String> {
    for entry in std::fs::read_dir(psbts_dir)? {
        let entry = entry?;
        let mut path = entry.path();
        path.push("psbt.json");
        if let Ok(psbt_json) = read_psbt_json(&path) {
            if let Ok((_, psbt)) = psbt_from_base64(&psbt_json.psbt) {
                if &psbt.global.unsigned_tx.txid() == txid {
                    return Ok(psbt_json.name);
                }
            }
        }
    }
    let mut count = 0usize;
    let mut psbts_name = psbts_dir.clone();
    psbts_name.push("dummy");
    loop {
        let name = format!("psbt-{}", count);
        psbts_name.set_file_name(&name);
        if !psbts_name.exists() {
            return Ok(name);
        }
        count += 1;
    }
}

/// psbts_dir is general psbts dir, name is extracted from PSBT
/// if file exists a PSBT merge will be attempted
pub fn save_psbt(
    psbt: &mut PSBT,
    psbts_dir: &mut PathBuf,
    qr_version: i16,
) -> Result<(PathBuf, Vec<PathBuf>)> {
    let name = get_psbt_name(psbt).unwrap_or_else(|| {
        let new_name = get_name(&psbts_dir, &psbt.global.unsigned_tx.txid()).unwrap(); // TODO remove unwrap
        info!("PSBT without name, giving one: {}", new_name);
        let pair = raw::Pair {
            key: get_name_key(),
            value: new_name.as_bytes().to_vec(),
        };
        let _ = psbt.global.insert_pair(pair);
        new_name
    });

    psbts_dir.push(&name);
    if psbts_dir.exists() {
        let mut old_psbt = psbts_dir.clone();
        old_psbt.push("psbt.json");
        if let Ok(old_psbt) = read_psbt(&old_psbt) {
            info!("old psbt exist, merging together");
            psbt.merge(old_psbt)?;
        }
    } else {
        fs::create_dir_all(&psbts_dir)?;
    }

    let (psbt_bytes, psbt_base64) = psbt_to_base64(psbt);
    info!("save_psbt name {:?} dir {:?}", name, psbts_dir);

    let psbt_json = PsbtJson {
        name,
        psbt: psbt_base64,
    };
    psbts_dir.push("psbt.json");
    let psbt_file = psbts_dir.clone();
    let contents = serde_json::to_string_pretty(&psbt_json)?;
    fs::write(&psbts_dir, contents.as_bytes())?;

    psbts_dir.set_file_name("qr");
    let qrs = save_qrs(psbt_bytes, psbts_dir.clone(), qr_version)?;

    Ok((psbt_file, qrs))
}

impl PSBTSigner {
    fn new(
        psbt: &PSBT,
        xprv: &ExtendedPrivKey,
        network: Network,
        derivations: u32,
        psbts_dir: PathBuf,
        allow_any_derivations: bool,
    ) -> Result<Self> {
        let exception = network == Network::Regtest && xprv.network == Network::Testnet;
        if !(network == xprv.network || exception) {
            return Err(
                "Master key network is different from the network passed through cli".into(),
            );
        }
        let secp = Secp256k1::signing_only();

        Ok(PSBTSigner {
            psbt: psbt.clone(),
            psbts_dir,
            xprv: *xprv,
            secp,
            derivations,
            network,
            allow_any_derivations,
        })
    }

    fn from_opt(opt: &SignOptions, network: Network) -> Result<Self> {
        let psbt = read_psbt(&opt.psbt_file)?;
        let psbt_file = opt.psbt_file.clone();
        let psbts_dir = psbt_file.parent().unwrap().parent().unwrap().to_path_buf(); //TODO remove unwrap

        let xprv_json = read_key(&opt.key)?;

        let signer = PSBTSigner::new(
            &psbt,
            &xprv_json.xprv,
            network,
            opt.total_derivations,
            psbts_dir,
            opt.allow_any_derivations,
        )?;
        Ok(signer)
    }

    pub fn sign(&mut self) -> Result<SignResult> {
        let initial_inputs = self.psbt.inputs.clone();
        let added_paths = self.init_hd_keypath_if_absent()?;

        for (i, input) in self.psbt.inputs.clone().iter().enumerate() {
            debug!("{} {:?}", i, input);
            let is_segwit = input.witness_utxo.is_some();

            match input.non_witness_utxo.as_ref() {
                None => {
                    return Err(
                        "non_witness_utxo must be Some, even for segwit due to the fee bug".into(),
                    )
                }
                Some(non_witness_utxo) => {
                    let prevout = self.psbt.global.unsigned_tx.input[i].previous_output;
                    if non_witness_utxo.txid() != prevout.txid {
                        return Err("prevout doesn't match non_witness_utxo".into());
                    }
                    if is_segwit {
                        let witness_utxo = input
                            .clone()
                            .witness_utxo
                            .expect("both witness_utxo and non_witness_utxo are none");
                        let script = match input.clone().redeem_script {
                            Some(script) => {
                                if witness_utxo.script_pubkey != script.to_p2sh() {
                                    return Err("witness_utxo script_pubkey doesn't match the redeem script converted to p2sh".into());
                                }
                                script
                            }
                            None => witness_utxo.script_pubkey,
                        };
                        if script.is_v0_p2wpkh() {
                            let script = to_p2pkh(&script.as_bytes()[2..]);
                            if !script.is_p2pkh() {
                                return Err("it is not a p2pkh script".into());
                            }
                            self.sign_input(&script, i)?;
                        } else {
                            let wit_script = input
                                .clone()
                                .witness_script
                                .expect("witness_script is none");
                            if script != wit_script.to_v0_p2wsh() {
                                return Err(
                                    "script and witness script to v0 p2wsh doesn't match".into()
                                );
                            }
                            self.sign_input(&wit_script, i)?;
                        }
                    } else {
                        let script_pubkey = non_witness_utxo.output[prevout.vout as usize]
                            .clone()
                            .script_pubkey;
                        match input.redeem_script.clone() {
                            Some(redeem_script) => {
                                if script_pubkey != redeem_script.to_p2sh() {
                                    return Err("script_pubkey does not match the redeem script converted to p2sh".into());
                                }
                                self.sign_input(&redeem_script, i)?;
                            }
                            None => {
                                self.sign_input(&script_pubkey, i)?;
                            }
                        };
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
        let is_segwit = input.witness_utxo.is_some();
        let my_fing = self.xprv.fingerprint(&self.secp);

        for (pubkey, (fing, child)) in input.hd_keypaths.iter() {
            if fing != &my_fing {
                continue;
            }
            if !self.allow_any_derivations {
                let path_slice = child.as_ref();
                if path_slice.len() != 2 {
                    return Err(format!("{} only two derivation paths allowed", child).into());
                } else {
                    if !(path_slice[0] == 0.into() || path_slice[0] == 1.into()) {
                        return Err(
                            format!("{} first derivation must be Soft 0 or 1", child).into()
                        );
                    }
                }
            }
            let privkey = self.xprv.derive_priv(&self.secp, &child)?;
            let derived_pubkey =
                secp256k1::PublicKey::from_secret_key(&self.secp, &privkey.private_key.key);
            if pubkey.key != derived_pubkey {
                return Err(
                    "pubkey derived and expected differs even if fingerprint matches!".into(),
                );
            }
            let (hash, sighash);
            if is_segwit {
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

    fn save_signed_psbt_file(&mut self, qr_version: i16) -> Result<(PathBuf, Vec<PathBuf>)> {
        save_psbt(&mut self.psbt, &mut self.psbts_dir.clone(), qr_version)
    }

    fn pretty_print(&self, wallets: &[WalletJson]) -> Result<PsbtPrettyPrint> {
        pretty_print(&self.psbt, self.network, wallets)
    }
}

pub fn start(opt: &SignOptions, network: Network) -> Result<PsbtPrettyPrint> {
    let wallet = read_wallet(&opt.wallet_descriptor_file)?;
    let mut psbt_signer = PSBTSigner::from_opt(opt, network)?;
    debug!("{:?}", psbt_signer);
    //TODO refuse to sign if my address has first level different from 0/1 and more than one level?
    let sign_result = psbt_signer.sign()?;
    let mut psbt_print = psbt_signer.pretty_print(&vec![wallet])?;

    if sign_result.added_paths {
        psbt_print.info.push("Added paths".to_string());
    }
    if sign_result.signed {
        let (psbt_file, _) = psbt_signer.save_signed_psbt_file(opt.qr_version)?;
        psbt_print.psbt_file = psbt_file;
        psbt_print.info.push("Added signatures".to_string());
    } else {
        psbt_print.info.push("No signature added".to_string());
    }

    Ok(psbt_print)
}

pub fn read_key(path: &PathBuf) -> Result<PrivateMasterKey> {
    let is_key = path
        .file_name()
        .ok_or_else(|| Error::Generic("no file_name".into()))?
        .to_str()
        .ok_or_else(|| Error::Generic("OsStr".into()))?
        == "PRIVATE.json";
    if !is_key {
        return Err(Error::Generic("private name MUST be PRIVATE.json".into()));
    }
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
    use crate::offline::sign::*;
    use crate::{psbt_from_base64, psbt_to_base64, PsbtJson, PSBT};
    use bitcoin::util::bip32::ExtendedPubKey;
    use bitcoin::Transaction;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempdir::TempDir;

    fn test_sign(
        psbt_to_sign: &mut PSBT,
        psbt_signed: &PSBT,
        xprv: &ExtendedPrivKey,
    ) -> Result<()> {
        let temp_dir = TempDir::new("test_sign").unwrap().into_path();
        let mut psbt_signer =
            PSBTSigner::new(psbt_to_sign, xprv, xprv.network, 10, temp_dir, true)?;
        psbt_signer.sign()?;

        assert_eq!(
            psbt_to_base64(&psbt_signer.psbt).1,
            psbt_to_base64(psbt_signed).1
        );
        assert_eq!(&psbt_signer.psbt, psbt_signed);
        Ok(())
    }

    fn perc_diff_with_core(psbt: &PSBT, core: usize) -> Result<bool> {
        let esteem = (estimate_weight(psbt)? / 4) as f64;
        let core = core as f64;
        let perc = ((esteem - core) / esteem).abs();
        Ok(perc < 0.1) // TODO reduce this 10% by improving estimation of the bip tx
    }

    fn extract_psbt(bytes: &[u8]) -> (Vec<u8>, PSBT) {
        let psbt_json: PsbtJson = serde_json::from_slice(bytes).unwrap();
        psbt_from_base64(&psbt_json.psbt).unwrap()
    }

    #[test]
    fn test_compression() {
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.json");
        let (psbt_ser, _) = extract_psbt(bytes);
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(&psbt_ser).unwrap();
        let compressed_bytes = e.finish().unwrap();
        assert_eq!(psbt_ser.len(), 903);
        assert_eq!(compressed_bytes.len(), 722);

        let bytes = include_bytes!("../../test_data/sign/psbt_bip.signed.json");
        let (psbt_ser, _) = extract_psbt(bytes);
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(&psbt_ser).unwrap();
        let compressed_bytes = e.finish().unwrap();
        assert_eq!(psbt_ser.len(), 1583);
        assert_eq!(compressed_bytes.len(), 1192);
    }

    #[test]
    fn test_psbt() {
        let secp = Secp256k1::signing_only();
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.signed.json");
        let (_, psbt_signed) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_bip.json");
        let (_, mut psbt_to_sign) = extract_psbt(bytes);

        let bytes = include_bytes!("../../test_data/sign/psbt_bip.key");
        let key: crate::PrivateMasterKey = serde_json::from_slice(bytes).unwrap();

        let tx1 = "0200000001aad73931018bd25f84ae400b68848be09db706eac2ac18298babee71ab656f8b0000000048473044022058f6fc7c6a33e1b31548d481c826c015bd30135aad42cd67790dab66d2ad243b02204a1ced2604c6735b6393e5b41691dd78b00f0c5942fb9f751856faa938157dba01feffffff0280f0fa020000000017a9140fb9463421696b82c833af241c78c17ddbde493487d0f20a270100000017a91429ca74f8a08f81999428185c97b5d852e4063f618765000000";
        let tx1: Transaction = deserialize(&hex::decode(tx1).unwrap()).unwrap();
        assert_eq!(
            format!("{}", tx1.txid()),
            "75ddabb27b8845f5247975c8a5ba7c6f336c4570708ebe230caf6db5217ae858"
        );

        let tx2 = "0200000000010158e87a21b56daf0c23be8e7070456c336f7cbaa5c8757924f545887bb2abdd7501000000171600145f275f436b09a8cc9a2eb2a2f528485c68a56323feffffff02d8231f1b0100000017a914aed962d6654f9a2b36608eb9d64d2b260db4f1118700c2eb0b0000000017a914b7f5faf40e3d40a5a459b1db3535f2b72fa921e88702483045022100a22edcc6e5bc511af4cc4ae0de0fcd75c7e04d8c1c3a8aa9d820ed4b967384ec02200642963597b9b1bc22c75e9f3e117284a962188bf5e8a74c895089046a20ad770121035509a48eb623e10aace8bfd0212fdb8a8e5af3c94b0b133b95e114cab89e4f7965000000";
        let tx2: Transaction = deserialize(&hex::decode(tx2).unwrap()).unwrap();
        assert_eq!(
            format!("{}", tx2.txid()),
            "1dea7cd05979072a3578cab271c02244ea8a090bbb46aa680a65ecd027048d83"
        );

        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );

        assert_eq!(
            bitcoin::consensus::serialize(&psbt_to_sign.global).len(),
            158
        );
        let inputs_len: usize = psbt_to_sign
            .inputs
            .iter()
            .map(|i| bitcoin::consensus::serialize(i).len())
            .sum();
        assert_eq!(inputs_len, 634);
        let outputs_len: usize = psbt_to_sign
            .outputs
            .iter()
            .map(|o| bitcoin::consensus::serialize(o).len())
            .sum();
        assert_eq!(outputs_len, 106);

        assert_eq!(
            bitcoin::consensus::serialize(&psbt_signed.global).len(),
            158
        );
        let inputs_len: usize = psbt_signed
            .inputs
            .iter()
            .map(|i| bitcoin::consensus::serialize(i).len())
            .sum();
        assert_eq!(inputs_len, 1314);
        let outputs_len: usize = psbt_signed
            .outputs
            .iter()
            .map(|o| bitcoin::consensus::serialize(o).len())
            .sum();
        assert_eq!(outputs_len, 106);

        assert!(
            test_sign(&mut psbt_to_sign, &psbt_signed, &key.xprv).is_err(),
            "segwit input missing previous tx"
        );
        psbt_to_sign.inputs[1].non_witness_utxo = psbt_to_sign.inputs[0].non_witness_utxo.clone();
        assert!(
            test_sign(&mut psbt_to_sign, &psbt_signed, &key.xprv).is_err(),
            "tx non matching prevout"
        );

        psbt_to_sign.inputs[1].non_witness_utxo = Some(tx2);
        test_sign(&mut psbt_to_sign, &psbt_signed, &key.xprv).unwrap();

        assert!(perc_diff_with_core(&psbt_to_sign, 462).unwrap()); // 462 is estimated_vsize from analyzepsbt

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.signed.json");
        let (_, mut psbt_1) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.json");
        let (_, orig) = extract_psbt(bytes);
        let mut psbt_to_sign = orig.clone();
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.key");
        let key: crate::PrivateMasterKey = serde_json::from_slice(bytes).unwrap();
        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );
        assert!(
            test_sign(&mut psbt_to_sign, &psbt_1, &key.xprv).is_err(),
            "segwit input missing previous tx"
        );
        let tx_in = "020000000001019e60071916a88cf0f5b9c6f015b7f8eef3ab1ef6ca4929b7236ec74e693f36210000000023220020c3af1472a85b23206da9be4fbef18d0ce5fd965671110d722a816e892d2e5f33fdffffff02801a0600000000002200201148e93e9315e37dbed2121be5239257af35adc03ffdfc5d914b083afa44dab80e07a1010000000017a9142aaba9f43085c5a6f28b0d01a8ed4dbcc0e5ec4f87040047304402203fdaeafde5fc1d1838d4c431abf6672f4cfee996f932187b31a4e3dad04d7b9f0220247d2cee5aabceb029ee6a1809a821fd95aa3ff02627977cbac8d00ff5a4628901473044022026879e4c65462161e2805ca26d392b0aace13906ec5b4776cac99f5e2bfd49f4022072500f1e2818a6738c37b6cedb2fd0a16375df34ce145e3f8fbfef7b7bec99d401475221020ca0e815748c41087075f3840c1edd9400f4db031dbe948b1929b6a93c72386a21026471f666489f80aed63bbbdee4f09ffcd69b40900435633cef5f5a35bf00932752ae4ff21700";
        let tx_in: Transaction = deserialize(&hex::decode(tx_in).unwrap()).unwrap();
        psbt_to_sign.inputs[0].non_witness_utxo = Some(tx_in.clone());
        test_sign(&mut psbt_to_sign, &psbt_1, &key.xprv).unwrap();
        assert!(perc_diff_with_core(&psbt_to_sign, 192).unwrap());

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.signed.json");
        let (_, psbt_2) = extract_psbt(bytes);
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.2.key");
        let mut psbt_to_sign = orig.clone();
        let key: crate::PrivateMasterKey = serde_json::from_slice(bytes).unwrap();
        assert_eq!(
            key.xpub.to_string(),
            ExtendedPubKey::from_private(&secp, &key.xprv).to_string()
        );
        assert!(
            test_sign(&mut psbt_to_sign, &psbt_2, &key.xprv).is_err(),
            "segwit input missing previous tx"
        );
        psbt_to_sign.inputs[0].non_witness_utxo = Some(tx_in);
        test_sign(&mut psbt_to_sign, &psbt_2, &key.xprv).unwrap();

        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.signed.json");
        let (psbt_complete_bytes, psbt_complete) = extract_psbt(bytes);

        psbt_1.merge(psbt_2).unwrap();

        assert_eq!(psbt_to_base64(&psbt_1).1, psbt_to_base64(&psbt_complete).1);
        assert_eq!(psbt_1, psbt_complete);
        assert_eq!(
            psbt_to_base64(&psbt_1).1,
            base64::encode(&psbt_complete_bytes)
        );
    }
}
