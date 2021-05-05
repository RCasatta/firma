use crate::list::ListOptions;
use crate::offline::decrypt::decrypt;
use crate::offline::descriptor::{derive_address, DeriveAddressOptions};
use crate::offline::sign::MessageToSign;
use crate::*;
use bitcoin::consensus::serialize;
use bitcoin::secp256k1::{Secp256k1, Signature};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, Fingerprint};
use bitcoin::util::key;
use bitcoin::{Address, Amount, Network, OutPoint, Script, SignedAmount, TxOut};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use structopt::StructOpt;

type HdKeypaths = BTreeMap<key::PublicKey, (Fingerprint, DerivationPath)>;

/// Print details regarding a Partially Signed Bitcoin Transaction (PSBT) given as parameter.
/// A `psbt_file` or a `psbt_base` should be specified.
#[derive(StructOpt, Debug, Serialize, Deserialize)]
#[structopt(name = "firma")]
pub struct PrintOptions {
    /// PSBT json file
    #[structopt(long)]
    pub psbt_file: Option<PathBuf>,

    /// PSBT as base64 string
    #[structopt(long)]
    pub psbt_base64: Option<String>,

    /// PSBT name contained in firma datadir
    #[structopt(long)]
    pub psbt_name: Option<String>,
}

impl OfflineContext {
    pub fn print(&self, opt: &PrintOptions) -> Result<PsbtPrettyPrint> {
        let psbt =
            match (&opt.psbt_file, &opt.psbt_base64, &opt.psbt_name) {
                (Some(path), None, None) => {
                    let psbt_json: Psbt = decrypt(path, &self.encryption_key)?;
                    psbt_json.psbt()?
                }
                (None, Some(base64), None) => psbt_from_base64(base64)?.1,
                (None, None, Some(name)) => {
                    let psbt_json: Psbt = self.read(name)?;
                    psbt_json.psbt()?
                }
                (None, None, None) => {
                    return Err("`psbt_file` or `psbt_base64` or `psbt_name` must be set".into())
                }
                _ => return Err(
                    "exactly one between `psbt_file`, `psbt_base64`, `psbt_name` must be specified"
                        .into(),
                ),
            };
        let kind = Kind::Wallet;
        let opt = ListOptions { kind };
        let result = self.list(&opt)?;
        let output = pretty_print(&psbt, self.network, &result.wallets)?;
        Ok(output)
    }
}

pub fn pretty_print(
    psbt: &BitcoinPsbt,
    network: Network,
    wallets: &[Wallet],
) -> Result<PsbtPrettyPrint> {
    let mut result = PsbtPrettyPrint::default();
    let mut previous_outputs: Vec<TxOut> = vec![];
    let mut output_values: Vec<u64> = vec![];
    let tx = &psbt.global.unsigned_tx;
    let vouts: Vec<OutPoint> = tx.input.iter().map(|el| el.previous_output).collect();
    for (i, input) in psbt.inputs.iter().enumerate() {
        let previous_output = match (&input.non_witness_utxo, &input.witness_utxo) {
            (_, Some(val)) => val,
            (Some(prev_tx), None) => {
                let outpoint = *vouts.get(i).ok_or(Error::MissingOutpoint)?;
                assert_eq!(prev_tx.txid(), outpoint.txid);
                prev_tx
                    .output
                    .get(outpoint.vout as usize)
                    .ok_or(Error::MissingTxout)?
            }
            _ => return Err("witness_utxo and non_witness_utxo are both None".into()),
        };
        previous_outputs.push(previous_output.clone());
    }
    let input_values: Vec<u64> = previous_outputs.iter().map(|o| o.value).collect();
    let mut balances = HashMap::new();
    let mut message_to_sign = MessageToSign::new(&psbt);
    let secp = Secp256k1::verification_only();

    for (i, input) in tx.input.iter().enumerate() {
        let addr = Address::from_script(&previous_outputs[i].script_pubkey, network)
            .ok_or(Error::NonDefaultScript)?;
        let keypaths = &psbt.inputs[i].bip32_derivation;
        let signatures: HashSet<Fingerprint> = psbt.inputs[i]
            .partial_sigs
            .iter()
            .filter(|(pk, signature)| {
                // Verify the signature in the PSBT is valid
                //TODO works only for v0_p2wsh
                let script = psbt.inputs[i].witness_script.as_ref().unwrap();
                let (_, message) = message_to_sign.hash(i, script).unwrap();
                let signature = Signature::from_der(&signature[..signature.len() - 1]).unwrap(); // remove sig_hash_type
                match secp.verify(&message, &signature, &pk.key) {
                    Ok(_) => true,
                    Err(_) => {
                        result.info.push(
                            "Signatures: one or more signature in the psbt is not valid"
                                .to_string(),
                        );
                        false
                    }
                }
            })
            .filter_map(|(k, _)| keypaths.get(k).map(|v| v.0))
            .collect();

        let wallet_if_any = wallet_with_path(keypaths, &wallets, &addr);
        if let Some((wallet, _)) = &wallet_if_any {
            *balances.entry(wallet.clone()).or_insert(0i64) -= previous_outputs[i].value as i64
        }
        let txin = entities::TxIn {
            outpoint: input.previous_output.to_string(),
            signatures,
            common: TxCommonInOut {
                value: Amount::from_sat(previous_outputs[i].value).to_string(),
                wallet_with_path: wallet_if_any.map(|(w, p)| format!("[{}]{}", w, p)),
            },
        };
        result.inputs.push(txin);
    }

    for (i, output) in tx.output.iter().enumerate() {
        let addr =
            Address::from_script(&output.script_pubkey, network).ok_or(Error::NonDefaultScript)?;
        let keypaths = &psbt.outputs[i].bip32_derivation;
        let wallet_if_any = wallet_with_path(keypaths, &wallets, &addr);
        if let Some((wallet, _)) = &wallet_if_any {
            *balances.entry(wallet.clone()).or_insert(0i64) += output.value as i64
        }
        let txout = entities::TxOut {
            address: addr.to_string(),
            common: TxCommonInOut {
                value: Amount::from_sat(output.value).to_string(),
                wallet_with_path: wallet_if_any.map(|(w, p)| format!("[{}]{}", w, p)),
            },
        };
        result.outputs.push(txout);
        output_values.push(output.value);
    }
    let balances_vec: Vec<String> = balances
        .iter()
        .map(|(k, v)| format!("{}: {}", k, SignedAmount::from_sat(*v).to_string()))
        .collect();
    result.balances = balances_vec.join("\n");

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
    let estimated_tx_vbytes = estimate_weight(psbt).ok().map(|e| e / 4);
    let estimated_fee_rate = estimated_tx_vbytes.map(|e| fee as f64 / e as f64);

    result.size = Size {
        estimated: estimated_tx_vbytes,
        unsigned: tx_vbytes,
        psbt: serialize(psbt).len(),
    };
    result.fee = Fee {
        absolute: fee,
        absolute_fmt: Amount::from_sat(fee).to_string(),
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
    SCRIPT_TYPE_FN.iter().position(|f| f(script))
}

/// returns a wallet name and a derivation iif the address parameter is the same as the one derived from the wallet
fn wallet_with_path(
    hd_keypaths: &HdKeypaths,
    wallets: &[Wallet],
    address: &Address,
) -> Option<(String, DerivationPath)> {
    for wallet in wallets {
        for (_, (finger, path)) in hd_keypaths.iter() {
            if wallet.fingerprints().contains(finger) {
                let path_vec: Vec<ChildNumber> = path.clone().into();
                let len = path_vec.len();
                if let ChildNumber::Normal { index } = path_vec.get(len - 2)? {
                    let descriptor = match index {
                        0 => &wallet.descriptor,
                        _ => return None,
                    };
                    if let ChildNumber::Normal { index } = path_vec.last()? {
                        let opts = DeriveAddressOptions {
                            descriptor: descriptor.to_string(),
                            index: *index,
                        };
                        if let Ok(derived) = derive_address(address.network, &opts) {
                            if &derived.address == address {
                                return Some((wallet.id.name.clone(), path.clone()));
                            }
                        }
                    }
                };
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::offline::print::{biggest_dividing_pow, pretty_print, script_type};
    use crate::{psbt_from_base64, Psbt, Wallet};
    use bitcoin::Network;

    #[test]
    fn test_biggest_dividing_pow() {
        assert_eq!(biggest_dividing_pow(3), 0);
        assert_eq!(biggest_dividing_pow(10), 1);
        assert_eq!(biggest_dividing_pow(11), 0);
        assert_eq!(biggest_dividing_pow(110), 1);
        assert_eq!(biggest_dividing_pow(1100), 2);
        assert_eq!(biggest_dividing_pow(1100030), 1);
    }

    #[test]
    fn test_script_type() {
        macro_rules! hex_script (($s:expr) => (bitcoin::blockdata::script::Script::from(::hex::decode($s).unwrap())));

        let s =
            hex_script!("21021aeaf2f8638a129a3156fbe7e5ef635226b0bafd495ff03afe2c843d7e3a4b51ac");
        assert_eq!(script_type(&s), Some(0usize));

        let s = hex_script!("76a91402306a7c23f3e8010de41e9e591348bb83f11daa88ac");
        assert_eq!(script_type(&s), Some(1usize));

        let s = hex_script!("a914acc91e6fef5c7f24e5c8b3f11a664aa8f1352ffd87");
        assert_eq!(script_type(&s), Some(2usize));

        let s = hex_script!("00140c3e2a4e0911aac188fe1cba6ef3d808326e6d0a");
        assert_eq!(script_type(&s), Some(3usize));

        let s = hex_script!("00201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f894921989");
        assert_eq!(script_type(&s), Some(4usize));
    }

    #[test]
    fn test_pretty_print() {
        let (_, to_carol_psbt) = psbt_from_base64("cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAM/AVmaXJtYQBuYW1lCHRvLWNhcm9sAAEAoQIAAAABG7mL63lJDPOLQybsXY8WZhK8QMjvz5D/qM6KBtZAYmQAAAAAIyIAIPynXT2ph1cCtzZ2E+fD0d6vmuZPc8BQvMyVxOjcK+c1/f///wJMiwYAAAAAABepFGdxKLPj9gk9IONcwMW/kz2S7YYIh6TOAAAAAAAAIgAg9ZFXIhxr0C/u7qGjb+y5bdnmVPnY3tH583t2S8HyPqp+hR0AAQErpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qgEFR1IhApKznFtt8+fKlGOcjgKzwmEgy8O2et7atlNfdA5bb80uIQN9dFnXvgcdA4fmLWblwKJbuzazugS3dzc6PrlDq2fd4FKuIgYCkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4couvgTjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gHB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAAAEBR1IhAuOCnowHNpvquGET8SUCHqm7lSymqSslu2U4B2VdZ9hAIQOo4hJeqVo5DnlJPz/2YUn3odyLWIHI1GBOEbzdokJRf1KuIgIC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAcouvgTjAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/HB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAEAAAAA").unwrap();
        let wallet = Wallet::new("wsh(multi(2,[a2ebe04e/48'/1'/0'/2']tpubDEXDRpvW2srXCSjAvC36zYkSE3jxT1wf7JXDo35Ln4NZpmaMNhq8o9coH9U9BQ5bAN4WDGxXV9d426iYKGorFF5wvv4Wv63cZsCotiXGGkD/0/*,[1f5e43d8/48'/1'/0'/2']tpubDFU4parcXvV8tBYt4rS4a8rGNF1DA32DCnRfhzVL6b3MSiDomV95rv9mb7W7jAPMTohyEYpbhVS8FbmTsuQsFRxDWPJX2ZFEeRPMFz3R1gh/0/*))#szg2xsau", Network::Testnet);
        let name = wallet.id.name.clone();

        let result = pretty_print(&to_carol_psbt, Network::Testnet, &[wallet]).unwrap();
        assert_eq!(format!("{}: -0.00052381 BTC", name), result.balances);

        assert_eq!("Privacy: outputs have different script types https://en.bitcoin.it/wiki/Privacy#Sending_to_a_different_script_type", result.info[0]);
        assert_eq!("Privacy: outputs have different precision https://en.bitcoin.it/wiki/Privacy#Round_numbers", result.info[1]);

        assert_eq!(result.fee.absolute, 381);

        dbg!(result);
    }

    #[test]
    fn test_pretty_print_wrong_sig() {
        let bytes = include_bytes!("../../test_data/sign/psbt_testnet.1.signed.json");
        let psbt_json: Psbt = serde_json::from_slice(bytes).unwrap();
        let (_, mut psbt) = psbt_from_base64(&psbt_json.psbt).unwrap();
        let result = pretty_print(&psbt, Network::Testnet, &[]).unwrap();
        assert_eq!(result.inputs.len(), 1);
        let signatures: usize = result.inputs.iter().map(|i| i.signatures.len()).sum();
        assert_eq!(signatures, 1);

        // changing 1 byte in the signature
        (*psbt.inputs[0].partial_sigs.iter_mut().next().unwrap().1)[10] += 1;
        let result = pretty_print(&psbt, Network::Testnet, &[]).unwrap();
        assert_eq!(result.inputs.len(), 1);
        let signatures: usize = result.inputs.iter().map(|i| i.signatures.len()).sum();
        assert_eq!(signatures, 0);
        assert!(result
            .info
            .contains(&"Signatures: one or more signature in the psbt is not valid".to_string()));
    }
}
