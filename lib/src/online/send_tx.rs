use crate::*;
use bitcoin::consensus::deserialize;
use bitcoin::Transaction;
use bitcoincore_rpc::RpcApi;
use log::{debug, info};
use std::path::PathBuf;

#[derive(structopt::StructOpt, Debug)]
pub struct SendTxOptions {
    /// filename containing the PSBT
    #[structopt(long = "psbt-file")]
    pub psbts_file: Vec<PathBuf>,

    /// filename containing the PSBT
    #[structopt(long = "psbt")]
    pub psbts: Vec<String>,

    /// broadcast transaction through the node, by default it is not broadcasted
    #[structopt(long)]
    pub broadcast: bool,
}

impl SendTxOptions {
    fn validate(&self) -> Result<()> {
        if self.psbts.is_empty() && self.psbts_file.is_empty() {
            return Err("At least one psbt is mandatory".into());
        }
        Ok(())
    }
}

impl Wallet {
    pub fn send_tx(&self, opt: &SendTxOptions) -> Result<SendTxOutput> {
        opt.validate()?;
        let mut psbts = vec![];
        for psbt_file in opt.psbts_file.iter() {
            let json = read_psbt_json(psbt_file)?;
            psbts.push(json.psbt);
        }
        psbts.extend(opt.psbts.clone());

        // Bitcoin core doesn't accept PSBT inputs with both witness_utxo and non_witness_utxo that we added
        // to prevent the fee bug, so we strip the non_witness_utxo if a witness_utxo is also present
        let mut psbts_stripped = vec![];
        for psbt_string in psbts.iter() {
            let mut psbt = psbt_from_base64(psbt_string)?.1;
            for input in psbt.inputs.iter_mut() {
                if input.non_witness_utxo.is_some() && input.witness_utxo.is_some() {
                    debug!("removing non witness for input {:?}", input);
                    input.non_witness_utxo = None;
                }
            }
            psbts_stripped.push(psbt_to_base64(&psbt).1);
        }

        let combined = self.client.combine_psbt(&psbts_stripped)?;
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

        Ok(send_tx)
    }
}
