use crate::offline::print::pretty_print;
use crate::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct ListOptions {
    /// list wallets, keys or psbts
    #[structopt(short, long)]
    pub kind: Kind,

    /// Return wallets only if wallet signature file is present and signature verifies
    #[structopt(long)]
    pub verify_wallets_signatures: bool,

    /// Optional encryption keys to read encrypted [PrivateMasterKey]
    #[structopt(skip)]
    pub encryption_keys: Vec<StringEncoding>,
}

impl Context {
    pub fn list(&self, opt: &ListOptions) -> Result<ListOutput> {
        let mut path = self.base()?;
        path.push(opt.kind.dir());
        let mut list = ListOutput::default();

        if path.is_dir() {
            debug!("listing {:?}", path);
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name().unwrap().to_str().unwrap();
                match opt.kind {
                    Kind::Wallet => {
                        //let secp = Secp256k1::verification_only();
                        let wallet: Result<WalletJson> = self.read(name);
                        match wallet {
                            Ok(wallet) => {
                                debug!("read {:?}", wallet.id.name);
                                let _wallet_path = path.clone();
                                let wallet_output = CreateWalletOutput {
                                    wallet,
                                    wallet_file: path.clone(),
                                    signature: None,
                                };
                                list.wallets.push(wallet_output);
                                /*
                                path.set_file_name("signature.json");
                                let signature_path = path.clone();

                                if !opt.verify_wallets_signatures {
                                    list.wallets.push(wallet_output);
                                } else {
                                    match verify_wallet_internal(&wallet_path, &signature_path, &secp) {
                                        Ok(result) => {
                                            if result.verified {
                                                wallet_output.signature = Some(result.signature);
                                                list.wallets.push(wallet_output)
                                            } else {
                                                warn!("signature doesn't match")
                                            }
                                        }
                                        Err(e) => warn!("wallet not added because {:?}", e),
                                    }
                                }

                                 */
                            }
                            Err(e) => {
                                warn!("Can't read wallet {:?}", e);
                            }
                        }
                    }
                    Kind::PSBT => {
                        let psbt_json: Result<PsbtJson> = self.read(name);
                        match psbt_json {
                            Ok(psbt_json) => {
                                let (_, psbt) = psbt_from_base64(&psbt_json.psbt)?;
                                let pretty = pretty_print(&psbt, self.network, &[])?;
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
                    Kind::MasterSecret => {
                        let key: Result<MasterSecretJson> = self.read(name);
                        match key {
                            Ok(key) => {
                                let key = MasterKeyOutput {
                                    key,
                                    private_file: path.clone(),
                                    public_file: None,
                                };
                                list.keys.push(key);
                                debug!("key decrypted");
                                break;
                            }
                            Err(e) => {
                                debug!("Can't read key {:?} because {:?}", &path, e);
                            }
                        }
                        /*path.push("PRIVATE.json");
                        debug!("try to read key {:?}", path);
                        let keys_iter = opt.encryption_keys.iter().map(Option::Some);
                        for encryption_key in keys_iter.chain(once(None)) {
                            debug!("using encryption_key {:?}", encryption_key);
                            match read_key(&path, encryption_key) {
                                Ok(key) => {
                                    let key = MasterKeyOutput {
                                        key,
                                        private_file: path.clone(),
                                        public_file: None,
                                    };
                                    list.keys.push(key);
                                    debug!("key decrypted");
                                    break;
                                }
                                Err(e) => {
                                    debug!("Can't read key {:?} because {:?}", &path, e);
                                }
                            }
                        }

                         */
                    }
                    _ => unimplemented!(),
                }
            }
        }

        Ok(list)
    }
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

fn read_qrs(_path: &PathBuf) -> Result<Vec<PathBuf>> {
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use crate::common::json::identifier::Kind;
    use crate::common::list::ListOptions;
    use crate::offline::random::RandomOptions;
    use crate::Context;
    use bitcoin::Network;
    use tempfile::TempDir;

    #[test]
    fn test_list() {
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "list".to_string();
        let rand_opts = RandomOptions::new(key_name);
        let context = Context {
            network: Network::Testnet,
            firma_datadir: temp_dir_str,
        };
        let _key = context.create_key(&rand_opts).unwrap();

        let kind = Kind::MasterSecret;
        let opt = ListOptions {
            kind,
            encryption_keys: vec![],
            verify_wallets_signatures: false,
        };
        let result = context.list(&opt);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list
            .keys
            .iter()
            .any(|key| key.key.id.name == rand_opts.key_name));
    }
}
