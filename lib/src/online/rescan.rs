use crate::OnlineContext;
use bitcoincore_rpc::RpcApi;
use serde_json::Value;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RescanOptions {
    /// The name of the wallet to be created
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,

    /// Specify the block height from which doing a blockchain rescan (use 0 to start from the beginning)
    #[structopt(long)]
    pub start_from: usize,
}

impl OnlineContext {
    pub fn rescan(&self, opt: &RescanOptions) -> crate::Result<Value> {
        let (_a, b) = self
            .make_client(&opt.wallet_name)?
            .rescan_blockchain(Some(opt.start_from), None)?;
        Ok(b.ok_or(crate::Error::MissingRescanUpTo)?.into())
    }
}
