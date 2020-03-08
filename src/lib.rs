use bitcoin::Network;
use log::info;
use log::{Level, LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

mod error;

pub use error::Error;

type Result<R> = std::result::Result<R, Error>;

static LOGGER: SimpleLogger = SimpleLogger;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrivateMasterKeyJson {
    pub xpub: String,
    pub xpriv: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launches: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicMasterKeyJson {
    pub xpub: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub psbt: String,
    pub fee: f64,
    pub changepos: u32,
    pub signed_psbt: Option<String>,
    pub only_sigs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletJson {
    pub name: String,
    pub main_descriptor: String,
    pub change_descriptor: String,
    pub daemon_opts: DaemonOpts,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletIndexesJson {
    pub main: u32,
    pub change: u32,
}

#[derive(StructOpt, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DaemonOpts {
    /// Bitcoin node rpc url
    #[structopt(long)]
    pub url: String,

    /// Bitcoin node cookie file
    #[structopt(long)]
    pub cookie_file: PathBuf,
}

#[derive(StructOpt, Debug, Clone)]
pub struct Context {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    pub network: Network,

    /// Name of the wallet
    #[structopt(short, long)]
    pub wallet_name: String,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub firma_datadir: String,
}

// from https://stackoverflow.com/questions/54267608/expand-tilde-in-rust-path-idiomatically
fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if p.starts_with("~") {
        if p == Path::new("~") {
            dirs::home_dir()
        } else {
            dirs::home_dir().map(|mut h| {
                if h == Path::new("/") {
                    // Corner case: `h` root directory;
                    // don't prepend extra `/`, just drop the tilde.
                    p.strip_prefix("~").unwrap().to_path_buf()
                } else {
                    h.push(p.strip_prefix("~/").unwrap());
                    h
                }
            })
        }
    } else {
        Some(p.to_path_buf())
    }
}

pub fn init_logger(verbose: u8) {
    let level = match verbose {
        0 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
        .expect("cannot initialize logging");
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

impl Context {
    pub fn path_for(&self, what: &str) -> PathBuf {
        path_for(
            &self.firma_datadir,
            self.network,
            Some(&self.wallet_name),
            what,
        )
    }

    pub fn save_wallet(&self, wallet: &WalletJson) -> Result<()> {
        let path = self.path_for("descriptor");
        if path.exists() {
            return Err("wallet already exist, I am not going to overwrite".into());
        }
        info!("Saving wallet data in {:?}", path);

        fs::write(path, serde_json::to_string_pretty(wallet)?)?;
        Ok(())
    }

    pub fn save_index(&self, indexes: &WalletIndexesJson) -> Result<()> {
        let path = self.path_for("indexes");
        info!("Saving index data in {:?}", path);
        fs::write(path, serde_json::to_string_pretty(indexes)?)?;

        Ok(())
    }

    pub fn load_wallet_and_index(&self) -> Result<(WalletJson, WalletIndexesJson)> {
        let wallet_path = self.path_for("descriptor");
        let wallet = fs::read(wallet_path)?;
        let wallet = serde_json::from_slice(&wallet)?;

        let indexes_path = self.path_for("indexes");
        let indexes = fs::read(indexes_path)?;
        let indexes = serde_json::from_slice(&indexes)?;

        Ok((wallet, indexes))
    }
}

pub fn generate_key_filenames(
    datadir: &str,
    network: Network,
    key_name: &str,
) -> Result<(PathBuf, PathBuf)> {
    let private_file = path_for(&datadir, network, None, &format!("{}-PRIVATE", key_name));
    let public_file = path_for(&datadir, network, None, &format!("{}-public", key_name));
    if private_file.exists() || public_file.exists() {
        return Err(format!(
            "{:?} or {:?} already exists, exiting to avoid unwanted override. Run --help.",
            &private_file, &public_file,
        )
        .into());
    }

    Ok((private_file, public_file))
}

fn save(value: String, output: &PathBuf) {
    fs::write(output, value).expect(&format!("Unable to write {:?}", output));
    info!("Saving {:?}", output);
}

pub fn save_public(public_key: &PublicMasterKeyJson, output: &PathBuf) {
    save(serde_json::to_string_pretty(public_key).unwrap(), output);
}

pub fn save_private(private_key: &PrivateMasterKeyJson, output: &PathBuf) {
    save(serde_json::to_string_pretty(private_key).unwrap(), output);
}

pub fn read_psbt(path: &Path) -> PsbtJson {
    let json = fs::read_to_string(path).unwrap();
    serde_json::from_str(&json).unwrap()
}

impl From<PrivateMasterKeyJson> for PublicMasterKeyJson {
    fn from(private: PrivateMasterKeyJson) -> Self {
        PublicMasterKeyJson { xpub: private.xpub }
    }
}

fn path_for(datadir: &str, network: Network, wallet_name: Option<&str>, what: &str) -> PathBuf {
    let mut path = PathBuf::from(datadir);
    path = expand_tilde(path).unwrap();
    path.push(format!("{}", network));
    if let Some(wallet_name) = wallet_name {
        path.push(wallet_name);
    }
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    path.push(&format!("{}.json", what));
    path
}
