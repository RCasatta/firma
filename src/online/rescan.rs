use bitcoincore_rpc::RpcApi;
use serde_json::Value;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RescanOptions {
    /// Specify the block height from which doing a blockchain rescan (default: 0)
    #[structopt(long)]
    pub start_from: Option<usize>,
}

impl crate::Wallet {
    pub fn rescan(&self, opt: &RescanOptions) -> firma::Result<Value> {
        self.client.rescan_blockchain(opt.start_from, None)?;
        Ok(Value::default())
    }
}
