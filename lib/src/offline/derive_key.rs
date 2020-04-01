use crate::file::save_keys;
use crate::offline::sign::read_key;
use crate::{MasterKeyOutput, PrivateMasterKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::ChildNumber;
use bitcoin::Network;
use std::path::PathBuf;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "derive_key")]
pub struct DeriveKeyOptions {
    /// Name of the master^2 key
    #[structopt(short, long)]
    from_key_file: PathBuf,

    /// Name of the generated master key, used as path to generate the child key
    #[structopt(short, long)]
    to_key_name: String,

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,
}

pub fn start(
    datadir: &str,
    network: Network,
    opt: &DeriveKeyOptions,
) -> crate::Result<MasterKeyOutput> {
    if opt.to_key_name.is_empty() {
        return Err("--to-key-name must have 1 or more characters".into());
    }
    let secp = Secp256k1::signing_only();
    let from_key_json = read_key(&opt.from_key_file)?;
    let mut child_key = from_key_json.xprv;
    let bytes = opt.to_key_name.as_bytes();
    for byte in bytes {
        let path = [ChildNumber::from_hardened_idx(*byte as u32)?];
        child_key = child_key.derive_priv(&secp, &path)?;
    }

    let child_key_json = PrivateMasterKey::from_xprv(child_key, &opt.to_key_name);
    let output = save_keys(
        datadir,
        network,
        &opt.to_key_name,
        child_key_json,
        opt.qr_version,
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::offline::derive_key::DeriveKeyOptions;
    use crate::offline::random::RandomOptions;
    use bitcoin::Network;
    use tempdir::TempDir;

    #[test]
    fn test_derive_key() -> crate::Result<()> {
        let temp_dir = TempDir::new("test_derive_key").unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());

        let key_name = "random".to_string();
        let rand_opts = RandomOptions::new(key_name);
        let key = crate::offline::random::create_key(&temp_dir_str, Network::Testnet, &rand_opts)
            .unwrap();

        let to_key_name = "derived".to_string();
        let mut der_opts = DeriveKeyOptions {
            from_key_file: key.private_file.clone(),
            to_key_name,
            qr_version: 14,
        };
        let derived =
            crate::offline::derive_key::start(&temp_dir_str, Network::Testnet, &der_opts.clone())
                .unwrap();

        assert_ne!(key.key, derived.key);

        let temp_dir_2 = TempDir::new("test_derive_key_2").unwrap();
        let temp_dir_str_2 = format!("{}/", temp_dir_2.path().display());
        let derived_2 =
            crate::offline::derive_key::start(&temp_dir_str_2, Network::Testnet, &der_opts)
                .unwrap();
        assert_eq!(derived.key, derived_2.key);

        der_opts.to_key_name = "".to_string();
        let key = crate::offline::derive_key::start(&temp_dir_str, Network::Testnet, &der_opts);
        assert!(key.is_err());
        assert_eq!(
            key.unwrap_err().to_string(),
            "--to-key-name must have 1 or more characters"
        );

        Ok(())
    }
}
