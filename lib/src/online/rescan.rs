use bitcoincore_rpc::RpcApi;
use serde_json::Value;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RescanOptions {
    /// Specify the block height from which doing a blockchain rescan (use 0 to start from the beginning)
    #[structopt(long)]
    pub start_from: usize,
}

impl crate::Wallet {
    pub fn rescan(&self, opt: &RescanOptions) -> crate::Result<Value> {
        let (_a, b) = self.client.rescan_blockchain(Some(opt.start_from), None)?;
        Ok(b.ok_or(crate::Error::MissingRescanUpTo)?.into())
    }
}
