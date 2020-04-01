use crate::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

pub mod balance;
pub mod create_tx;
pub mod create_wallet;
pub mod get_address;
pub mod list_coins;
pub mod rescan;
pub mod send_tx;

pub struct Wallet {
    pub client: Client,
    context: Context,
}

impl Wallet {
    pub fn new(url: String, auth: Auth, context: Context) -> Result<Self> {
        Ok(Wallet {
            client: Client::new(url, auth)?,
            context,
        })
    }
}

fn save_psbt(psbt: &PsbtJson, path: &PathBuf) -> Result<()> {
    if path.exists() {
        return Err(Error::FileExist(path.clone()));
    }
    info!("Saving psbt in {:?}", path);
    fs::write(&path, serde_json::to_string_pretty(psbt)?)?;
    Ok(())
}

fn read_xpubs_files(paths: &[PathBuf]) -> Result<Vec<ExtendedPubKey>> {
    let mut xpubs = vec![];
    for xpub_path in paths.iter() {
        let content = fs::read(xpub_path)?;
        let json: PublicMasterKey = serde_json::from_slice(&content)?;
        xpubs.push(json.xpub.clone());
    }
    Ok(xpubs)
}

impl Wallet {
    pub fn load_if_unloaded(&self, wallet_name: &str) -> Result<()> {
        match self.client.load_wallet(wallet_name) {
            Ok(_) => info!("wallet {} loaded", wallet_name),
            Err(e) => {
                if e.to_string().contains("not found") {
                    return Err(format!("{} not found in the bitcoin node", wallet_name).into());
                } else {
                    debug!("wallet {} already loaded", wallet_name);
                }
            }
        }
        Ok(())
    }
}
