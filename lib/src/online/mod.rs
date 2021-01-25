use crate::common::json::identifier::{Identifier, Kind};
use crate::*;
use bitcoin::util::bip32::ExtendedPubKey;
use structopt::StructOpt;

pub mod balance;
pub mod create_tx;
pub mod create_wallet;
pub mod get_address;
pub mod list_coins;
pub mod rescan;
pub mod send_tx;

#[derive(StructOpt, Debug)]
pub struct WalletNameOptions {
    /// The name of the wallet to use
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,
}

#[derive(StructOpt, Debug)]
pub struct ConnectOptions {
    #[structopt(flatten)]
    pub context: Context,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,
}

fn read_xpubs_names(names: &[String], context: &Context) -> Result<Vec<ExtendedPubKey>> {
    let mut result = vec![];
    for name in names {
        let k: PublicMasterKey = Identifier::new(context.network, Kind::DescriptorPublicKey, name)
            .read(&context.firma_datadir)?;
        result.push(k.xpub);
    }
    Ok(result)
}
