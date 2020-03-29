use crate::*;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Instruction::PushBytes;
use bitcoin::consensus::deserialize;
use bitcoin::util::key;
use bitcoin::{Network, Script};
use log::{info, LevelFilter, Metadata, Record};
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};

pub mod cmd;
pub mod error;
pub mod json;

static LOGGER: SimpleLogger = SimpleLogger;

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("log")
                .expect("can't open log");
            let mut stream = BufWriter::new(file);
            stream
                .write(format!("{} - {}\n", record.level(), record.args()).as_bytes())
                .expect("can't write log");
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .expect("cannot initialize logging");
}

impl From<PrivateMasterKey> for PublicMasterKey {
    fn from(private: PrivateMasterKey) -> Self {
        PublicMasterKey { xpub: private.xpub }
    }
}

pub fn generate_key_filenames(
    datadir: &str,
    network: Network,
    key_name: &str,
) -> Result<(PathBuf, PathBuf)> {
    let private_file = path_for(&datadir, network, None, &format!("{}-PRIVATE", key_name))?;
    let public_file = path_for(&datadir, network, None, &format!("{}-public", key_name))?;
    if private_file.exists() || public_file.exists() {
        return Err(format!(
            "{:?} or {:?} already exists, exiting to avoid unwanted override. Run --help.",
            &private_file, &public_file,
        )
        .into());
    }

    Ok((private_file, public_file))
}

fn save(value: String, output: &PathBuf) -> Result<()> {
    fs::write(output, value)?;
    info!("Saving {:?}", output);
    Ok(())
}

pub fn save_public(public_key: &PublicMasterKey, output: &PathBuf) -> Result<()> {
    save(serde_json::to_string_pretty(public_key)?, output)
}

pub fn save_private(private_key: &PrivateMasterKey, output: &PathBuf) -> Result<()> {
    save(serde_json::to_string_pretty(private_key)?, output)
}

pub fn save_keys(
    datadir: &str,
    network: Network,
    key_name: &str,
    key: PrivateMasterKey,
) -> Result<MasterKeyOutput> {
    let (private_key_file, public_key_file) = generate_key_filenames(datadir, network, key_name)?;
    save_private(&key, &private_key_file)?;
    save_public(&key.clone().into(), &public_key_file)?;

    Ok(MasterKeyOutput {
        key,
        public_file: public_key_file,
        private_file: private_key_file,
    })
}

pub fn read_psbt_json(path: &Path) -> Result<PsbtJson> {
    let json = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

pub fn read_psbt(path: &Path, only_ready: bool) -> Result<PSBT> {
    let psbt_json = read_psbt_json(&path)?;
    if !only_ready && (psbt_json.signed_psbt.is_some() || psbt_json.only_sigs.is_some()) {
        return Err(Error::AlreadySigned);
    }
    psbt_from_base64(&psbt_json.psbt)
}

pub fn path_for(
    datadir: &str,
    network: Network,
    wallet_name: Option<&str>,
    what: &str,
) -> Result<PathBuf> {
    let mut path = PathBuf::from(datadir);
    path = expand_tilde(path)?;
    path.push(format!("{}", network));
    if let Some(wallet_name) = wallet_name {
        path.push(wallet_name);
    }
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    path.push(&format!("{}.json", what));
    Ok(path)
}

pub fn psbt_from_base64(s: &str) -> Result<PSBT> {
    let bytes = base64::decode(s)?;
    let psbt = deserialize(&bytes)?;
    Ok(psbt)
}

pub fn estimate_weight(psbt: &PSBT) -> Result<usize> {
    let unsigned_weight = psbt.global.unsigned_tx.get_weight();
    let mut spending_weight = 0usize;

    for input in psbt.inputs.iter() {
        let (script, factor) = match (&input.redeem_script, &input.witness_script) {
            (Some(redeem_script), None) => (redeem_script, 4),
            (_, Some(witness_script)) => (witness_script, 1), // factor=1 for segwit discount
            _ => return err("both redeem and witness script are None"),
        };
        //TODO signature are less in NofM where N<M
        let current = script.len() + expected_signatures(script)? * 72; // using 72 as average signature size
        spending_weight += current * factor;
    }

    Ok(unsigned_weight + spending_weight)
}

fn expected_signatures(script: &Script) -> Result<usize> {
    let bytes = script.as_bytes();
    Ok(
        if bytes.last().ok_or_else(fn_err("script empty"))?
            == &opcodes::all::OP_CHECKMULTISIG.into_u8()
        {
            read_pushnum(bytes[0])
                .map(|el| el as usize)
                .unwrap_or(0usize)
        } else {
            extract_pub_keys(script)?.len()
        },
    )
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

pub fn extract_pub_keys(script: &Script) -> Result<Vec<key::PublicKey>> {
    let mut result = vec![];
    for instruct in script.iter(false) {
        if let PushBytes(a) = instruct {
            if a.len() == 33 {
                result.push(key::PublicKey::from_slice(&a)?);
            }
        }
    }
    Ok(result)
}

// from https://stackoverflow.com/questions/54267608/expand-tilde-in-rust-path-idiomatically
pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Result<PathBuf> {
    let p = path_user_input.as_ref();
    if p.starts_with("~") {
        let mut home_dir = dirs::home_dir().ok_or_else(fn_err("cannot retrieve home dir"))?;
        if p == Path::new("~") {
            Ok(home_dir)
        } else {
            if home_dir == Path::new("/").to_path_buf() {
                // Corner case: `home_dir` root directory;
                // don't prepend extra `/`, just drop the tilde.
                Ok(p.strip_prefix("~")?.to_path_buf())
            } else {
                home_dir.push(p.strip_prefix("~/")?);
                Ok(home_dir)
            }
        }
    } else {
        Ok(p.to_path_buf())
    }
}

// #[cfg(test)]
pub fn throw_if_err(result: &Result<serde_json::Value>) -> Result<()> {
    match result {
        Ok(value) => {
            if let Some(serde_json::Value::String(error)) = value.get("error") {
                return err(error);
            }
        }
        Err(e) => return err(&e.to_string()),
    }
    Ok(())
}
