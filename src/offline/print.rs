use bitcoin::util::bip32::{DerivationPath, Fingerprint};
use bitcoin::util::key;
use bitcoin::{Address, Network, OutPoint, Script, TxOut};
use firma::*;
use serde_json::{to_value, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use structopt::StructOpt;

type HDKeypaths = BTreeMap<key::PublicKey, (Fingerprint, DerivationPath)>;

/// Sign a Partially Signed Bitcoin Transaction (PSBT) with a key.
#[derive(StructOpt, Debug)]
#[structopt(name = "firma")]
pub struct PrintOptions {
    /// PSBT json file
    psbt_file: PathBuf,

    /// File containing the wallet descriptor, show if outputs are mine.
    #[structopt(short, long, parse(from_os_str))]
    wallet_descriptor_file: PathBuf,
}

pub fn start(opt: &PrintOptions, network: Network) -> Result<Value> {
    let psbt = read_psbt(&opt.psbt_file, true)?;
    let wallet = read_wallet(&opt.wallet_descriptor_file)?;
    Ok(to_value(pretty_print(
        &psbt,
        network,
        &wallet.fingerprints,
    )?)?)
}

pub fn pretty_print(
    psbt: &PSBT,
    network: Network,
    fingerprints: &HashSet<Fingerprint>,
) -> Result<PsbtPrettyPrint> {
    let mut result = PsbtPrettyPrint::default();
    let mut previous_outputs: Vec<TxOut> = vec![];
    let mut output_values: Vec<u64> = vec![];
    let tx = &psbt.global.unsigned_tx;
    let vouts: Vec<OutPoint> = tx.input.iter().map(|el| el.previous_output).collect();
    for (i, input) in psbt.inputs.iter().enumerate() {
        let previous_output = match (&input.non_witness_utxo, &input.witness_utxo) {
            (Some(prev_tx), None) => {
                let outpoint = *vouts.get(i).ok_or_else(fn_err("can't find outpoint"))?;
                assert_eq!(prev_tx.txid(), outpoint.txid);
                prev_tx
                    .output
                    .get(outpoint.vout as usize)
                    .ok_or_else(fn_err("can't find txout"))?
            }
            (None, Some(val)) => val,
            _ => return err("witness_utxo and non_witness_utxo are both None or both Some"),
        };
        previous_outputs.push(previous_output.clone());
    }
    let input_values: Vec<u64> = previous_outputs.iter().map(|o| o.value).collect();

    for (i, input) in tx.input.iter().enumerate() {
        result.inputs.push(format!(
            "#{} {} ({}) {}",
            i,
            input.previous_output,
            derivation_paths(&psbt.inputs[i].hd_keypaths),
            previous_outputs[i].value,
        ));
    }

    for (i, output) in tx.output.iter().enumerate() {
        result.outputs.push(format!(
            "#{} {} {} ({}{}) {}",
            i,
            hex::encode(&output.script_pubkey.as_bytes()),
            Address::from_script(&output.script_pubkey, network)
                .ok_or_else(fn_err("non default script"))?,
            derivation_paths(&psbt.outputs[i].hd_keypaths),
            is_mine(&psbt.outputs[i].hd_keypaths, &fingerprints),
            output.value
        ));
        output_values.push(output.value);
    }

    // Privacy analysis
    // Detect different script types in the outputs
    let mut script_types = HashSet::new();
    for o in tx.output.iter() {
        script_types.insert(script_type(&o.script_pubkey));
    }
    if script_types.len() > 1 {
        result.info.push("Privacy: outputs have different script types https://en.bitcoin.it/wiki/Privacy#Sending_to_a_different_script_type".to_string());
    }

    // Detect rounded amounts
    let divs: Vec<u8> = tx
        .output
        .iter()
        .map(|o| biggest_dividing_pow(o.value))
        .collect();
    if let (Some(max), Some(min)) = (divs.iter().max(), divs.iter().min()) {
        if max - min >= 3 {
            result.info.push("Privacy: outputs have different precision https://en.bitcoin.it/wiki/Privacy#Round_numbers".to_string());
        }
    }

    // Detect unnecessary input heuristic
    if previous_outputs.len() > 1 {
        if let Some(smallest_input) = input_values.iter().min() {
            if output_values.iter().any(|value| value < smallest_input) {
                result.info.push("Privacy: smallest output is smaller then smallest input https://en.bitcoin.it/wiki/Privacy#Unnecessary_input_heuristic".to_string());
            }
        }
    }

    // Detect script reuse
    let input_scripts: HashSet<Script> = previous_outputs
        .iter()
        .map(|o| o.script_pubkey.clone())
        .collect();
    if tx
        .output
        .iter()
        .any(|o| input_scripts.contains(&o.script_pubkey))
    {
        result.info.push(
            "Privacy: address reuse https://en.bitcoin.it/wiki/Privacy#Address_reuse".to_string(),
        );
    }

    let fee = input_values.iter().sum::<u64>() - output_values.iter().sum::<u64>();
    let tx_vbytes = tx.get_weight() / 4;
    let estimated_tx_vbytes = estimate_weight(psbt)? / 4;
    let estimated_fee_rate = fee as f64 / estimated_tx_vbytes as f64;

    result.sizes = Size {
        estimated: estimated_tx_vbytes,
        unsigned: tx_vbytes,
    };
    result.fee = Fee {
        absolute: fee,
        rate: estimated_fee_rate,
    };

    Ok(result)
}

fn biggest_dividing_pow(num: u64) -> u8 {
    let mut start = 10u64;
    let mut count = 0u8;
    loop {
        if num % start != 0 {
            return count;
        }
        start *= 10;
        count += 1;
    }
}

const SCRIPT_TYPE_FN: [fn(&Script) -> bool; 5] = [
    Script::is_p2pk,
    Script::is_p2pkh,
    Script::is_p2sh,
    Script::is_v0_p2wpkh,
    Script::is_v0_p2wsh,
];
fn script_type(script: &Script) -> Option<usize> {
    for (i, f) in SCRIPT_TYPE_FN.iter().enumerate() {
        if f(script) {
            return Some(i);
        }
    }
    return None;
}

pub fn derivation_paths(hd_keypaths: &HDKeypaths) -> String {
    let mut vec: Vec<String> = hd_keypaths
        .iter()
        .map(|(_, (_, p))| format!("{:?}", p))
        .collect();
    vec.sort();
    vec.dedup();
    vec.join(", ")
}

fn is_mine(hd_keypaths: &HDKeypaths, wallet: &HashSet<Fingerprint>) -> String {
    if !hd_keypaths.is_empty() && hd_keypaths.iter().all(|(_, (f, _))| wallet.contains(f)) {
        " MINE"
    } else {
        ""
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::print::biggest_dividing_pow;

    #[test]
    fn test_biggest_dividing_pow() {
        assert_eq!(biggest_dividing_pow(3), 0);
        assert_eq!(biggest_dividing_pow(10), 1);
        assert_eq!(biggest_dividing_pow(11), 0);
        assert_eq!(biggest_dividing_pow(110), 1);
        assert_eq!(biggest_dividing_pow(1100), 2);
        assert_eq!(biggest_dividing_pow(1100030), 1);
    }
}
