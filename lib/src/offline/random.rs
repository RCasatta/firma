use crate::*;
use bitcoin::Network;
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

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    #[serde(default)]
    pub qr_version: i16,

    /// Optional encryption key for saving the key file encrypted
    /// in CLI it is populated from standard input
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

impl RandomOptions {
    pub fn new(key_name: String) -> Self {
        RandomOptions {
            key_name,
            qr_version: 20,
            encryption_key: None,
        }
    }
}

pub fn create_key(datadir: &str, network: Network, opt: &RandomOptions) -> Result<MasterKeyOutput> {
    let sec = rand::thread_rng().gen::<[u8; 32]>();
    let mnemonic = Mnemonic::new(&sec)?;
    let master_key = PrivateMasterKey::new(network, &mnemonic, &opt.key_name)?;
    let output = save_keys(
        datadir,
        network,
        &opt.key_name,
        master_key,
        opt.qr_version,
        opt.encryption_key.as_ref(),
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::offline::random::{self, RandomOptions};
    use bitcoin::Network;
    use tempfile::TempDir;

    #[test]
    fn test_random() {
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "random".to_string();
        let rand_opts_1 = RandomOptions::new(key_name);
        let key_1 = random::create_key(&temp_dir_str, Network::Testnet, &rand_opts_1).unwrap();
        let result = random::create_key(&temp_dir_str, Network::Testnet, &rand_opts_1);
        assert!(result.is_err(), "Overwrite a key");
        assert!(result.unwrap_err().to_string().contains("already exist"));

        let key_name = "random_2".to_string();
        let rand_opts_2 = RandomOptions::new(key_name);
        let key_2 = random::create_key(&temp_dir_str, Network::Testnet, &rand_opts_2).unwrap();

        assert_ne!(key_1, key_2);
        assert_ne!(key_1.key, key_2.key);
        assert_ne!(key_1.public_file, key_2.public_file);
        assert_ne!(key_1.private_file, key_2.private_file);
    }
}
