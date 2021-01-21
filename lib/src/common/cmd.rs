use crate::common::json::identifier::{IdKind, Identifier};
use crate::*;
use bitcoincore_rpc::{Auth, Client};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DaemonOpts {
    /// Bitcoin node rpc url
    #[structopt(long)]
    pub url: String,

    /// Bitcoin node cookie file
    #[structopt(long)]
    pub cookie_file: PathBuf,
}

impl DaemonOpts {
    pub fn make_client(&self, wallet: Option<String>) -> Result<Client> {
        let url = match wallet {
            Some(wallet) => format!("{}/wallet/{}", self.url, wallet),
            None => self.url.to_string(),
        };
        Ok(Client::new(
            url,
            Auth::CookieFile(self.cookie_file.clone()),
        )?)
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct Context {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    pub network: bitcoin::Network,

    /// Name of the wallet
    #[structopt(short, long)]
    pub wallet_name: String, //TOTO remove, use NewContext

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub firma_datadir: String,
}

#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
pub struct NewContext {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    pub network: bitcoin::Network,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub firma_datadir: String,
}

impl NewContext {
    pub fn read<T>(&self, kind: IdKind, name: &str) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug,
    {
        Ok(Identifier::new(self.network, kind, name).read(&self.firma_datadir)?)
    }
}

impl Context {
    fn path_builder_for(&self, kind: Kind, name: Option<String>) -> PathBuilder {
        PathBuilder::new(&self.firma_datadir, self.network, kind, name)
    }

    pub fn path_for_qr(&self, kind: Kind, name: Option<String>) -> Result<PathBuf> {
        self.path_builder_for(kind, name).file("qr")
    }

    pub fn path_for_wallet_qr(&self) -> Result<PathBuf> {
        self.path_for_qr(Kind::Wallet, Some(self.wallet_name.to_string()))
    }

    pub fn filename_for_wallet(&self, name: &str) -> Result<PathBuf> {
        self.path_builder_for(Kind::Wallet, Some(self.wallet_name.to_string()))
            .file(name)
    }

    pub fn psbts_dir(&self) -> Result<PathBuf> {
        self.path_builder_for(Kind::PSBT, None).type_path()
    }

    pub fn save_wallet(&self, wallet: &WalletJson) -> Result<PathBuf> {
        let path = self.filename_for_wallet("descriptor.json")?;
        if path.exists() {
            return Err(Error::FileExist(path));
        }
        info!("Saving wallet data in {:?}", &path);
        fs::write(&path, serde_json::to_string_pretty(wallet)?)?;
        Ok(path)
    }

    pub fn save_signature(&self, wallet: &WalletSignatureJson) -> Result<PathBuf> {
        let path = self.filename_for_wallet("signature.json")?;
        if path.exists() {
            return Err(Error::FileExist(path));
        }
        info!("Saving wallet signature data in {:?}", &path);
        fs::write(&path, serde_json::to_string_pretty(wallet)?)?;
        Ok(path)
    }

    pub fn save_index(&self, indexes: &IndexesJson) -> Result<()> {
        let path = self.filename_for_wallet("indexes.json")?;
        info!("Saving index data in {:?}", path);
        fs::write(path, serde_json::to_string_pretty(indexes)?)?;
        Ok(())
    }

    pub fn save_daemon_opts(&self, daemon_opts: &DaemonOpts) -> Result<()> {
        let path = self.filename_for_wallet("daemon_opts.json")?;
        info!("Saving daemon_opts data in {:?}", path);
        fs::write(path, serde_json::to_string_pretty(daemon_opts)?)?;
        Ok(())
    }

    pub fn decrease_index(&self) -> Result<()> {
        let (_, mut indexes) = self.load_wallet_and_index()?;
        indexes.main -= 1;
        self.save_index(&indexes)?;
        Ok(())
    }

    pub fn load_daemon_opts(&self) -> Result<DaemonOpts> {
        let mut path = expand_tilde(&self.firma_datadir)?;
        path.push(self.network.to_string());
        path.push("daemon_opts.json");
        let daemon_opts_bytes = std::fs::read(&path)
            .map_err(|e| crate::Error::FileNotFoundOrCorrupt(path, e.to_string()))?;
        let daemon_opts: DaemonOpts = serde_json::from_slice(&daemon_opts_bytes)?;
        Ok(daemon_opts)
    }

    // TODO many times called only for one file, split?
    /// load the wallet and related indexes and daemon opts
    pub fn load_wallet_and_index(&self) -> Result<(WalletJson, IndexesJson)> {
        let wallet_path = self.filename_for_wallet("descriptor.json")?;
        debug!("load wallet: {:?}", wallet_path);
        let wallet = read_wallet(&wallet_path)
            .map_err(|e| Error::FileNotFoundOrCorrupt(wallet_path.clone(), e.to_string()))?;

        let indexes_path = self.filename_for_wallet("indexes.json")?;
        debug!("load indexes: {:?}", indexes_path);
        let indexes = read_indexes(&indexes_path)
            .map_err(|e| Error::FileNotFoundOrCorrupt(wallet_path.clone(), e.to_string()))?;

        Ok((wallet, indexes))
    }
}
