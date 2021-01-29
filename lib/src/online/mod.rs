use crate::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod balance;
pub mod create_tx;
pub mod create_wallet;
pub mod get_address;
pub mod list_coins;
pub mod rescan;
pub mod send_tx;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct WalletNameOptions {
    /// The name of the wallet to use
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,
}

impl From<&str> for WalletNameOptions {
    fn from(name: &str) -> Self {
        WalletNameOptions {
            wallet_name: name.to_string(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct PathOptions {
    #[structopt(short, long)]
    pub path: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct ConnectOptions {
    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,
}
