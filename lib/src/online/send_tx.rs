use crate::*;
use bitcoin::consensus::deserialize;
use bitcoin::Transaction;
use bitcoincore_rpc::RpcApi;
use log::{debug, info};

#[derive(structopt::StructOpt, Debug)]
pub struct SendTxOptions {
    /// The name of the wallet to use to broadcast the tx
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,

    /// names containing the PSBTs
    #[structopt(long = "psbt-name")]
    pub psbts_name: Vec<String>,

    /// the PSBTs content as base64
    #[structopt(long = "psbt")]
    pub psbts: Vec<String>,

    /// broadcast transaction through the node, by default it is not broadcasted
    #[structopt(long)]
    pub broadcast: bool,
}

impl SendTxOptions {
    fn validate(&self) -> Result<()> {
        if self.psbts.is_empty() && self.psbts_name.is_empty() {
            return Err("At least one psbt is mandatory".into());
        }
        Ok(())
    }
}

impl OnlineContext {
    pub fn send_tx(&self, opt: &SendTxOptions) -> Result<SendTxOutput> {
        opt.validate()?;
        let client = self.make_client(&opt.wallet_name)?;
        let mut psbts = vec![];
        for psbt_name in opt.psbts_name.iter() {
            let json: Psbt = self.read(psbt_name)?;
            psbts.push(json.psbt);
        }
        psbts.extend(opt.psbts.clone());

        let combined = client.combine_psbt(&psbts)?;
        debug!("combined {:?}", combined);

        let finalized = client.finalize_psbt(&combined, Some(true))?;
        debug!("finalized {:?}", finalized);

        let bytes = finalized.hex.ok_or(Error::MissingHex)?;
        let hex = hex::encode(&bytes);

        let mut broadcasted = false;
        if opt.broadcast {
            let hash = client.send_raw_transaction(&bytes)?;
            broadcasted = true;
            info!("{:?}", hash);
        } else {
            info!("{}", hex);
        }

        let txid = deserialize::<Transaction>(&hex::decode(&hex)?)?.txid();
        let send_tx = SendTxOutput {
            hex,
            txid,
            broadcasted,
        };

        Ok(send_tx)
    }
}
