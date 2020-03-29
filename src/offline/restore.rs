use crate::Result;
use bitcoin::bech32::{self, FromBase32};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use firma::{err, save_keys, MasterKeyOutput, PrivateMasterKey};
use std::io;
use std::str::FromStr;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug)]
#[structopt(name = "restore")]
pub struct RestoreOptions {
    /// Name of the key
    #[structopt(short, long)]
    key_name: String,

    /// Kind of the secret material
    #[structopt(short, long)]
    nature: Nature,

    /// value of the secret component, could be xprv or seed in hex or bech32
    value: String,
}

#[derive(Debug)]
enum Nature {
    Xprv,
    HexSeed,
    Bech32Seed,
}

impl FromStr for Nature {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "xprv" => Ok(Nature::Xprv),
            "hex-seed" => Ok(Nature::HexSeed),
            "bech32-seed" => Ok(Nature::Bech32Seed),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("({}) valid values are: xprv, hex-seed, bech32-seed", s),
            )),
        }
    }
}

pub fn start(datadir: &str, network: Network, opt: &RestoreOptions) -> Result<MasterKeyOutput> {
    let master_key = match opt.nature {
        Nature::Xprv => ExtendedPrivKey::from_str(&opt.value)?.into(),
        Nature::Bech32Seed => {
            //TODO bech32 lib does not support error detection
            let (hrp, vec_u5) = bech32::decode(&opt.value)?;
            if hrp != "s" {
                return err(
                    "human readable part of the bech32 seed is wrong (must start with 's')",
                );
            }
            let sec = Vec::<u8>::from_base32(&vec_u5)?;
            PrivateMasterKey::new(network, &sec)?
        }
        Nature::HexSeed => {
            let sec = hex::decode(&opt.value)?;
            PrivateMasterKey::new(network, &sec)?
        }
    };

    let output = save_keys(datadir, network, &opt.key_name, master_key)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::random::RandomOptions;
    use crate::restore::{Nature, RestoreOptions};
    use bitcoin::Network;
    use tempdir::TempDir;

    #[test]
    fn test_restore() -> firma::Result<()> {
        let temp_dir = TempDir::new("test_restore").unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());
        let key_name_random = "test_restore_random".to_string();
        let rand_opts = RandomOptions {
            key_name: key_name_random,
        };
        let key_orig = crate::random::start(&temp_dir_str, Network::Testnet, &rand_opts).unwrap();

        let key_name_restored = "test_restore_restored".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Xprv,
            value: key_orig.key.xprv.to_string(),
        };
        let key_restored =
            crate::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key.xprv, key_restored.key.xprv);
        assert_eq!(key_orig.key.xpub, key_restored.key.xpub);

        let key_name_restored = "test_restore_seed".to_string();
        assert!(key_orig.key.seed.is_some());
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Bech32Seed,
            value: key_orig.key.seed.as_ref().unwrap().bech32.clone(),
        };
        let key_restored =
            crate::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key, key_restored.key);

        let key_name_restored = "test_restore_seed_hex".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::HexSeed,
            value: key_orig.key.seed.as_ref().unwrap().hex.clone(),
        };
        let key_restored =
            crate::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key, key_restored.key);

        let key_name_restored = "test_restore_fail".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::HexSeed,
            value: "X".to_string(),
        };
        let result = crate::restore::start(&temp_dir_str, Network::Testnet, &restore_opts);
        assert!(result.is_err());

        let key_name_restored = "test_restore_fail".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Xprv,
            value: key_orig.key.xpub.to_string(),
        };
        let result = crate::restore::start(&temp_dir_str, Network::Testnet, &restore_opts);
        assert!(result.is_err());

        Ok(())
    }
}
