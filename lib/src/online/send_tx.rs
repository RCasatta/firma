use crate::*;
use bitcoin::consensus::deserialize;
use bitcoin::Transaction;
use bitcoincore_rpc::RpcApi;
use log::{debug, info};
//use std::path::PathBuf;
use crate::common::json::identifier::{IdKind, Identifier};

#[derive(structopt::StructOpt, Debug)]
pub struct SendTxOptions {
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

impl Wallet {
    pub fn send_tx(&self, opt: &SendTxOptions) -> Result<SendTxOutput> {
        opt.validate()?;
        let mut psbts = vec![];
        for psbt_name in opt.psbts_name.iter() {
            let json: PsbtJson = Identifier::new(self.context.network, IdKind::PSBT, &psbt_name)
                .read(&self.context.firma_datadir)?;
            psbts.push(json.psbt);
        }
        psbts.extend(opt.psbts.clone());

        let combined = self.client.combine_psbt(&psbts)?;
        debug!("combined {:?}", combined);

        let finalized = self.client.finalize_psbt(&combined, Some(true))?;
        debug!("finalized {:?}", finalized);

        let bytes = finalized.hex.ok_or(Error::MissingHex)?;
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

        Ok(send_tx)
    }
}
