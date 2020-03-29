use crate::sign::read_key;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::ChildNumber;
use bitcoin::Network;
use firma::{err, save_keys, MasterKeyOutput, PrivateMasterKey};
use std::path::PathBuf;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "restore")]
pub struct DeriveKeyOptions {
    /// Name of the master^2 key
    #[structopt(short, long)]
    from_key_file: PathBuf,

    /// Name of the generated master key, used as path to generate the child key
    #[structopt(short, long)]
    to_key_name: String,
}

pub fn start(
    datadir: &str,
    network: Network,
    opt: &DeriveKeyOptions,
) -> crate::Result<MasterKeyOutput> {
    if opt.to_key_name.is_empty() {
        return err("--to-key-name must have 1 or more characters");
    }
    let secp = Secp256k1::signing_only();
    let from_key_json = read_key(&opt.from_key_file)?;
    let mut child_key = from_key_json.xprv.clone();
    let bytes = opt.to_key_name.as_bytes();
    for byte in bytes {
        let path = [ChildNumber::from_hardened_idx(*byte as u32)?];
        child_key = child_key.derive_priv(&secp, &path)?;
    }

    let child_key_json = PrivateMasterKey::from(child_key);
    let output = save_keys(datadir, network, &opt.to_key_name, child_key_json)?;

    Ok(output)
}


#[cfg(test)]
mod tests {
    use crate::random::RandomOptions;
    use bitcoin::Network;
    use tempdir::TempDir;
    use crate::derive_key::DeriveKeyOptions;

    #[test]
    fn test_derive_key() -> firma::Result<()> {
        let temp_dir = TempDir::new("test_derive_key").unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "random".to_string();
        let rand_opts = RandomOptions { key_name };
        let key = crate::random::start(&temp_dir_str, Network::Testnet, &rand_opts).unwrap();

        let to_key_name = "derived".to_string();
        let der_opts = DeriveKeyOptions { from_key_file: key.private_file.clone(), to_key_name};
        let derived = crate::derive_key::start(&temp_dir_str, Network::Testnet, &der_opts.clone()).unwrap();
        assert_ne!(key.key, derived.key);

        let temp_dir_2 = TempDir::new("test_derive_key_2").unwrap();
        let temp_dir_str_2 = format!("{}/", temp_dir_2.path().display());
        let derived_2 = crate::derive_key::start(&temp_dir_str_2, Network::Testnet, &der_opts).unwrap();
        assert_eq!(derived.key, derived_2.key);

        Ok(())
    }
}
