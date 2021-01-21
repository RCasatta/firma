use crate::common::json::identifier::{IdKind, Identifier};
use crate::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoincore_rpc::{Client, RpcApi};
use log::{debug, info};

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
    pub fn new(client: Client, context: Context) -> Self {
        Wallet { client, context }
    }
}

fn read_xpubs_names(names: &[String], context: &Context) -> Result<Vec<ExtendedPubKey>> {
    let mut result = vec![];
    for name in names {
        let k: PublicMasterKey =
            Identifier::new(context.network, IdKind::DescriptorPublicKey, name)
                .read(&context.firma_datadir)?;
        result.push(k.xpub);
    }
    Ok(result)
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
