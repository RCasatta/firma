use crate::offline::print::pretty_print;
use crate::offline::sign::read_key;
use crate::*;
use bitcoin::Network;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct ListOptions {
    /// list wallets, keys or psbts
    #[structopt(short, long)]
    pub kind: Kind,
}

pub fn list(datadir: &str, network: Network, opt: &ListOptions) -> Result<ListOutput> {
    let path = PathBuilder::new(datadir, network, opt.kind, None).type_path()?;
    let mut list = ListOutput::default();

    if path.is_dir() {
        debug!("listing {:?}", path);
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let mut path = entry.path();
            match opt.kind {
                Kind::Wallet => {
                    path.push("descriptor.json");
                    debug!("try to read wallet {:?}", path);
                    match read_wallet(&path) {
                        Ok(wallet) => {
                            let qr_files = read_qrs(&path)?;
                            let wallet = CreateWalletOutput {
                                qr_files, //TODO check if file exist?
                                wallet,
                                wallet_file: path.clone(),
                            };
                            list.wallets.push(wallet);
                        }
                        Err(e) => {
                            warn!("Can't read wallet {:?}", e);
                        }
                    }
                }
                Kind::PSBT => {
                    path.push("psbt.json");
                    debug!("try to read psbt {:?}", path);
                    match read_psbt_json(&path) {
                        Ok(psbt_json) => {
                            let (_, psbt) = psbt_from_base64(&psbt_json.psbt)?;
                            let pretty = pretty_print(&psbt, network, &[])?;
                            let qr_files = read_qrs(&path)?;
                            let psbt_out = PsbtJsonOutput {
                                psbt: psbt_json,
                                signatures: signatures_needed(&pretty.inputs),
                                unsigned_txid: psbt.global.unsigned_tx.txid(),
                                file: path.clone(),
                                qr_files,
                            };
                            list.psbts.push(psbt_out);
                        }
                        Err(e) => {
                            warn!("Can't read psbt {:?}", e);
                        }
                    }
                }
                Kind::Key => {
                    path.push("PRIVATE.json");
                    debug!("try to read key {:?}", path);
                    match read_key(&path) {
                        Ok(key) => {
                            let public_qr_files = read_qrs(&path)?;
                            let key = MasterKeyOutput {
                                key,
                                private_file: path.clone(),
                                public_file: None,
                                public_qr_files, //TODO populate if they exists
                            };
                            list.keys.push(key);
                        }
                        Err(e) => {
                            warn!("Can't read key {:?}", e);
                        }
                    }
                }
            }
        }
    }

    Ok(list)
}

fn signatures_needed(inputs: &[TxIn]) -> String {
    // TODO reasoning on the first input, should reason as a total?
    let number = inputs.first().map(|i| i.signatures.len()).unwrap_or(0);
    match number {
        0 => "No signatures".to_string(),
        1 => "1 signature".to_string(),
        n => format!("{} signatures", n),
    }
}

fn read_qrs(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut path = path.parent().expect("root has no parent").to_path_buf();
    path.push("qr");
    let mut vec = vec![];
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            vec.push(entry.path());
        }
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use crate::common::list::{list, ListOptions};
    use crate::offline::random::RandomOptions;
    use crate::Kind;
    use bitcoin::Network;
    use tempfile::TempDir;

    #[test]
    fn test_list() {
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "list".to_string();
        let rand_opts = RandomOptions::new(key_name);
        let _key = crate::offline::random::create_key(&temp_dir_str, Network::Testnet, &rand_opts)
            .unwrap();

        let kind = Kind::Key;
        let opt = ListOptions { kind };
        let result = list(&temp_dir_str, Network::Testnet, &opt);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list
            .keys
            .iter()
            .any(|key| key.key.name == rand_opts.key_name));
    }
}
