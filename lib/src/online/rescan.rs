use crate::OnlineContext;
use bitcoin::Network;
use bitcoincore_rpc::RpcApi;
use serde_json::Value;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RescanOptions {
    /// The name of the wallet to be created
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,

    /// Specify the block height from which doing a blockchain rescan (use 0 to start from the beginning)
    /// if missing will default to network segwit activation block
    #[structopt(long)]
    pub start_from: Option<usize>,
}

impl OnlineContext {
    pub fn rescan(&self, opt: &RescanOptions) -> crate::Result<Value> {
        let start_from = opt.start_from.unwrap_or({
            // segwit activation block
            match self.network {
                Network::Bitcoin => 481824,
                Network::Testnet => 834624,
                Network::Regtest => 0,
                Network::Signet => 0,
            }
        });

        let (_a, b) = self
            .make_client(&opt.wallet_name)?
            .rescan_blockchain(Some(start_from), None)?;
        Ok(b.ok_or(crate::Error::MissingRescanUpTo)?.into())
    }
}
