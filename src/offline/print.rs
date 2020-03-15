use bitcoin::Network;
use firma::*;
use std::path::PathBuf;
use structopt::StructOpt;
use log::info;
use bitcoin::Address;
use std::collections::BTreeMap;
use bitcoin::util::bip32::{DerivationPath, Fingerprint};
use bitcoin::util::key;


/// Sign a Partially Signed Bitcoin Transaction (PSBT) with a key.
#[derive(StructOpt, Debug)]
#[structopt(name = "firma")]
pub struct PrintOptions {
    /// PSBT json file
    psbt_file: PathBuf,
}

pub fn start(opt: &PrintOptions, network: Network) -> Result<()> {
    let psbt = read_psbt(&opt.psbt_file, true)?;
    pretty_print(&psbt, network)
}

pub fn pretty_print(psbt: &PSBT, network: Network) -> Result<()> {
    let mut input_values: Vec<u64> = vec![];
    let mut output_values: Vec<u64> = vec![];
    let tx = &psbt.global.unsigned_tx;
    let ins = tx.input.iter();
    let vouts: Vec<usize> = ins.map(|el| el.previous_output.vout as usize).collect();
    for (i, input) in psbt.inputs.iter().enumerate() {
        let val = match (&input.non_witness_utxo, &input.witness_utxo) {
            (Some(val), None) => {
                let vout = *vouts.get(i).ok_or_else(fn_err("can't find vout"))?;
                val.output
                    .get(vout)
                    .ok_or_else(fn_err("can't find value"))?
                    .value
            }
            (None, Some(val)) => val.value,
            _ => return err("witness_utxo and non_witness_utxo are both None or both Some"),
        };
        input_values.push(val);
    }

    info!("inputs [# prevout:vout value]:");
    for (i, input) in tx.input.iter().enumerate() {
        info!(
            "#{} {} ({}) {}",
            i,
            input.previous_output,
            derivation_paths(&psbt.inputs[i].hd_keypaths),
            input_values[i],
        );
    }
    info!("\noutputs [# script address amount]:");
    for (i, output) in tx.output.iter().enumerate() {
        // TODO calculate if it is mine
        info!(
            "#{} {} {} ({}) {}",
            i,
            hex::encode(&output.script_pubkey.as_bytes()),
            Address::from_script(&output.script_pubkey, network)
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown address".into()),
            derivation_paths(&psbt.outputs[i].hd_keypaths),
            output.value
        );
        output_values.push(output.value);
    }
    // TODO show privacy analysis like blockstream.info
    let fee = input_values.iter().sum::<u64>() - output_values.iter().sum::<u64>();

    let tx_vbytes = tx.get_weight() / 4;
    let estimated_tx_vbytes = estimate_weight(psbt)? / 4;
    let estimated_fee_rate = fee as f64 / estimated_tx_vbytes as f64;

    info!("");
    info!("absolute fee       : {:>6}   satoshi", fee);
    info!("unsigned tx        : {:>6}   vbyte", tx_vbytes);
    info!("estimated tx       : {:>6}   vbyte", estimated_tx_vbytes);
    info!("estimated fee rate : {:>8.1} sat/vbyte", estimated_fee_rate);
    Ok(())
}

pub fn derivation_paths(
    hd_keypaths: &BTreeMap<key::PublicKey, (Fingerprint, DerivationPath)>,
) -> String {
    let mut vec = vec![];
    for (_, (_, path)) in hd_keypaths.iter() {
        vec.push(format!("{:?}", path));
    }
    vec.sort();
    vec.dedup();
    vec.join(", ")
}
