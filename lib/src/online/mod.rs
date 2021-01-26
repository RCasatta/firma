use crate::*;
use std::path::PathBuf;
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
pub struct PathOptions {
    pub path: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct ConnectOptions {
    #[structopt(flatten)]
    pub context: Context,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,
}
