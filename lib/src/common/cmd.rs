use crate::*;
use log::{debug, info};
use serde::{Deserialize, Serialize};
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

#[derive(StructOpt, Debug, Clone)]
pub struct Context {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    pub network: bitcoin::Network,

    /// Name of the wallet
    #[structopt(short, long)]
    pub wallet_name: String,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub firma_datadir: String,
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

    pub fn save_signature(&self, wallet: &WalletSignature) -> Result<PathBuf> {
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
        let (_, mut indexes, _) = self.load_wallet_index_daemon()?;
        indexes.main -= 1;
        self.save_index(&indexes)?;
        Ok(())
    }

    // TODO many times called only for one file, split?
    /// load the wallet and related indexes and daemon opts
    pub fn load_wallet_index_daemon(&self) -> Result<(WalletJson, IndexesJson, DaemonOpts)> {
        let wallet_path = self.filename_for_wallet("descriptor.json")?;
        debug!("load wallet: {:?}", wallet_path);
        let wallet = read_wallet(&wallet_path)
            .map_err(|e| Error::FileNotFoundOrCorrupt(wallet_path.clone(), e.to_string()))?;

        let indexes_path = self.filename_for_wallet("indexes.json")?;
        debug!("load indexes: {:?}", indexes_path);
        let indexes = read_indexes(&indexes_path)
            .map_err(|e| Error::FileNotFoundOrCorrupt(wallet_path.clone(), e.to_string()))?;

        let daemon_opts_path = self.filename_for_wallet("daemon_opts.json")?;
        debug!("load daemon_opts: {:?}", daemon_opts_path);
        let daemon_opts = read_daemon_opts(&daemon_opts_path)
            .map_err(|e| Error::FileNotFoundOrCorrupt(daemon_opts_path.clone(), e.to_string()))?;

        Ok((wallet, indexes, daemon_opts))
    }
}
