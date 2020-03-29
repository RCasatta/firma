use crate::*;
use log::info;
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
    pub fn path_for(&self, what: &str) -> Result<PathBuf> {
        path_for(
            &self.firma_datadir,
            self.network,
            Some(&self.wallet_name),
            what,
        )
    }

    pub fn save_wallet(&self, wallet: &WalletJson) -> Result<PathBuf> {
        let path = self.path_for("descriptor")?;
        if path.exists() {
            return Err(Error::Generic(
                "wallet already exist, I am not going to overwrite".into(),
            ));
        }
        info!("Saving wallet data in {:?}", &path);

        fs::write(&path, serde_json::to_string_pretty(wallet)?)?;
        Ok(path)
    }

    pub fn save_index(&self, indexes: &WalletIndexes) -> Result<()> {
        let path = self.path_for("indexes")?;
        info!("Saving index data in {:?}", path);
        fs::write(path, serde_json::to_string_pretty(indexes)?)?;

        Ok(())
    }

    pub fn decrease_change_index(&self) -> Result<()> {
        let (_, mut indexes) = self.load_wallet_and_index()?;
        indexes.change -= 1;
        self.save_index(&indexes)?;
        Ok(())
    }

    pub fn load_wallet_and_index(&self) -> Result<(WalletJson, WalletIndexes)> {
        let wallet_path = self.path_for("descriptor")?;
        let wallet = read_wallet(&wallet_path)?;

        let indexes_path = self.path_for("indexes")?;
        let indexes = read_indexes(&indexes_path)?;

        Ok((wallet, indexes))
    }
}

pub fn read_wallet(path: &PathBuf) -> Result<WalletJson> {
    let wallet = fs::read(path)?;
    Ok(serde_json::from_slice(&wallet)?)
}

fn read_indexes(path: &PathBuf) -> Result<WalletIndexes> {
    let indexes = fs::read(path)?;
    Ok(serde_json::from_slice(&indexes)?)
}
