use crate::*;
use common::mnemonic::Mnemonic;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug, Serialize, Deserialize, Clone)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    pub key_name: String,

    /// Optional encryption key for saving the key file encrypted
    /// in CLI it is populated from standard input
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

impl RandomOptions {
    pub fn new(key_name: String) -> Self {
        RandomOptions {
            key_name,
            encryption_key: None,
        }
    }
}

impl Context {
    pub fn create_key(&self, opt: &RandomOptions) -> Result<MasterSecretJson> {
        let sec = rand::thread_rng().gen::<[u8; 32]>();
        let mnemonic = Mnemonic::new(&sec)?;
        let master_key = MasterSecretJson::new(self.network, &mnemonic, &opt.key_name)?;
        self.write_keys(&master_key)?;

        Ok(master_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::offline::random::RandomOptions;
    use crate::Context;
    use bitcoin::Network;
    use tempfile::TempDir;

    #[test]
    fn test_random() {
        let temp_dir = TempDir::new().unwrap();
        let key_name = "random".to_string();
        let rand_opts_1 = RandomOptions::new(key_name);
        let context = Context {
            firma_datadir: format!("{}/", temp_dir.path().display()),
            network: Network::Testnet,
        };
        let key_1 = context.create_key(&rand_opts_1).unwrap();
        let result = context.create_key(&rand_opts_1);
        assert!(result.is_err(), "Overwrite a key");
        assert!(result.unwrap_err().to_string().contains("Cannot overwrite"));

        let key_name = "random_2".to_string();
        let rand_opts_2 = RandomOptions::new(key_name);
        let key_2 = context.create_key(&rand_opts_2).unwrap();

        assert_ne!(key_1, key_2);
    }
}
