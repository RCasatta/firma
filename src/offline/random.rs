use bitcoin::Network;
use firma::*;
use rand::Rng;
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    pub key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &RandomOptions) -> Result<MasterKeyOutput> {
    let sec = rand::thread_rng().gen::<[u8; 16]>();
    let master_key = PrivateMasterKey::new(network, &sec)?;
    let output = save_keys(datadir, network, &opt.key_name, master_key)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::random::RandomOptions;
    use bitcoin::Network;
    use tempdir::TempDir;

    #[test]
    fn test_random() -> firma::Result<()> {
        let temp_dir = TempDir::new("test_random").unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "random".to_string();
        let rand_opts_1 = RandomOptions { key_name };
        let key_1 = crate::random::start(&temp_dir_str, Network::Testnet, &rand_opts_1).unwrap();
        let result = crate::random::start(&temp_dir_str, Network::Testnet, &rand_opts_1);
        assert!(result.is_err());

        let key_name = "random_2".to_string();
        let rand_opts_2 = RandomOptions { key_name };
        let key_2 = crate::random::start(&temp_dir_str, Network::Testnet, &rand_opts_2).unwrap();

        assert_ne!(key_1, key_2);
        assert_ne!(key_1.key, key_2.key);
        assert_ne!(key_1.public_file, key_2.public_file);
        assert_ne!(key_1.private_file, key_2.private_file);

        Ok(())
    }
}
