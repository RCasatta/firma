use log::{Level, LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

static LOGGER: SimpleLogger = SimpleLogger;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyJson {
    pub xpub: String,
    pub xpriv: String,
    pub launches: String,
    pub faces: u32,
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
pub struct WalletIndexes {
    pub main: u32,
    pub change: u32,
}

#[derive(StructOpt, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DaemonOpts {
    /// Bitcoin node rpc url
    #[structopt(long)]
    pub url: String,

    /// Bitcoin node rpc user
    #[structopt(long)]
    pub rpcuser: String,

    /// Bitcoin node rpc password
    #[structopt(long)]
    pub rpcpassword: String,
}

pub fn name_to_path(datadir: &str, wallet_name: &str, ext: &str) -> PathBuf {
    let mut path = PathBuf::from(datadir);
    path = expand_tilde(path).unwrap();
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    path.push(&format!("{}.{}", wallet_name, ext));
    path
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
