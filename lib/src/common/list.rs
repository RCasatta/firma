use crate::*;
use log::debug;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct ListOptions {
    /// list wallets, keys or psbts
    #[structopt(short, long)]
    pub kind: Kind,
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
                let name = path.file_name().unwrap().to_str().unwrap(); //TODO map err
                match opt.kind {
                    Kind::Wallet => {
                        debug!("read wallet jsons {:?}", name);
                        match self.read::<WalletJson>(name) {
                            Ok(wallet) => list.wallets.push(wallet),
                            Err(e) => debug!("can't read {} because {:?}", name, e),
                        }
                    }
                    Kind::WalletSignature => {
                        debug!("read wallet signature jsons {:?}", name);
                        match self.read::<WalletSignatureJson>(name) {
                            Ok(wallet_signature) => list.wallets_signatures.push(wallet_signature),
                            Err(e) => debug!("can't read {} because {:?}", name, e),
                        }
                    }
                    Kind::PSBT => {
                        debug!("read psbt json {:?}", name);
                        match self.read::<PsbtJson>(name) {
                            Ok(psbt_json) => list.psbts.push(psbt_json),
                            Err(e) => debug!("can't read {} because {:?}", name, e),
                        }
                    }
                    Kind::MasterSecret => {
                        debug!("read keys jsons {:?}", name);
                        match self.read::<MasterSecretJson>(name) {
                            Ok(secret_key) => list.master_secrets.push(secret_key),
                            Err(e) => debug!("can't read {} because {:?}", name, e),
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }

        Ok(list)
    }
}

fn _signatures_needed(inputs: &[TxIn]) -> String {
    // TODO reasoning on the first input, should reason as a total?
    let number = inputs.first().map(|i| i.signatures.len()).unwrap_or(0);
    match number {
        0 => "No signatures".to_string(),
        1 => "1 signature".to_string(),
        n => format!("{} signatures", n),
    }
}

#[cfg(test)]
mod tests {
    use crate::common::context::tests::TestContext;
    use crate::common::json::identifier::Kind;
    use crate::common::list::ListOptions;
    use crate::offline::random::RandomOptions;

    #[test]
    fn test_list() {
        let key_name = "list".to_string();
        let rand_opts = RandomOptions { key_name };
        let context = TestContext::new();
        let _key = context.create_key(&rand_opts).unwrap();

        let kind = Kind::MasterSecret;
        let opt = ListOptions { kind };
        let result = context.list(&opt);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list
            .master_secrets
            .iter()
            .any(|key| key.id.name == rand_opts.key_name));
    }
}
