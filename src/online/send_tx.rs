use crate::*;
use bitcoin::consensus::deserialize;
use bitcoin::Transaction;
use bitcoincore_rpc::RpcApi;
use log::{debug, info};
use serde_json::{to_value, Value};

#[derive(structopt::StructOpt, Debug)]
pub struct SendTxOptions {
    /// filename containing the PSBT
    #[structopt(long = "psbt")]
    pub psbts: Vec<PathBuf>,

    /// broadcast transaction through the node, by default it is not broadcasted
    #[structopt(long)]
    pub broadcast: bool,
}

impl SendTxOptions {
    fn validate(&self) -> Result<()> {
        if self.psbts.is_empty() {
            return firma::err("At least one psbt is mandatory");
        }
        Ok(())
    }
}

impl Wallet {
    pub fn send_tx(&self, opt: &SendTxOptions) -> Result<Value> {
        opt.validate()?;
        let mut psbts = vec![];
        for psbt_file in opt.psbts.iter() {
            let json = read_psbt_json(psbt_file)?;
            psbts.push(json.signed_psbt.expect("signed_psbt not found"));
        }
        let combined = self.client.combine_psbt(&psbts)?;
        debug!("combined {:?}", combined);

        let finalized = self.client.finalize_psbt(&combined, Some(true))?;
        debug!("finalized {:?}", finalized);

        let bytes = finalized.hex.ok_or_else(fn_err("hex is empty"))?;
        let hex = hex::encode(&bytes);

        let mut broadcasted = false;
        if opt.broadcast {
            let hash = self.client.send_raw_transaction(&bytes)?;
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

        Ok(to_value(send_tx)?)
    }
}
