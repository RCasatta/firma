use crate::{save_keys, MasterKeyOutput, PrivateMasterKey};
use crate::{Result, ToHrp};
use bitcoin::bech32::{self, FromBase32};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io;
use std::str::FromStr;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug, Deserialize, Serialize)]
#[structopt(name = "restore")]
pub struct RestoreOptions {
    /// Name of the key
    #[structopt(short, long)]
    key_name: String,

    /// Kind of the secret material
    #[structopt(short, long)]
    nature: Nature,

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,

    /// value of the secret component, could be xprv or seed in hex or bech32
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
    debug!("restore {:?}", &opt);
    let master_key = match opt.nature {
        Nature::Xprv => {
            PrivateMasterKey::from_xprv(ExtendedPrivKey::from_str(&opt.value)?, &opt.key_name)
        }
        Nature::Bech32Seed => {
            //TODO bech32 lib does not support error detection
            let (hrp, vec_u5) = bech32::decode(&opt.value)?;
            if hrp != network.to_hrp() {
                return Err(format!(
                    "in network {} bech32 seed must start with '{}'",
                    network,
                    network.to_hrp()
                )
                .into());
            }
            let sec = Vec::<u8>::from_base32(&vec_u5)?;
            PrivateMasterKey::new(network, &sec, &opt.key_name)?
        }
        Nature::HexSeed => {
            let sec = hex::decode(&opt.value)?;
            PrivateMasterKey::new(network, &sec, &opt.key_name)?
        }
    };

    let output = save_keys(datadir, network, &opt.key_name, master_key, opt.qr_version)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::offline::random::RandomOptions;
    use crate::offline::restore::{Nature, RestoreOptions};
    use bitcoin::Network;
    use tempdir::TempDir;

    #[test]
    fn test_restore() {
        let temp_dir = TempDir::new("test_restore").unwrap();
        let temp_dir_str = format!("{}/", temp_dir.path().display());
        let key_name_random = "test_restore_random".to_string();
        let rand_opts = RandomOptions::new(key_name_random);

        let key_orig =
            crate::offline::random::create_key(&temp_dir_str, Network::Testnet, &rand_opts)
                .unwrap();

        let key_name_restored = "test_restore_restored".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Xprv,
            value: key_orig.key.xprv.to_string(),
            qr_version: 14,
        };
        let key_restored =
            crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key.xprv, key_restored.key.xprv);
        assert_eq!(key_orig.key.xpub, key_restored.key.xpub);

        let key_name_restored = "test_restore_seed".to_string();
        assert!(key_orig.key.seed.is_some());
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Bech32Seed,
            value: key_orig.key.seed.as_ref().unwrap().bech32.clone(),
            qr_version: 14,
        };
        let key_restored =
            crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key.xprv, key_restored.key.xprv);

        let key_name_restored = "test_restore_seed_hex".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::HexSeed,
            value: key_orig.key.seed.as_ref().unwrap().hex.clone(),
            qr_version: 14,
        };
        let key_restored =
            crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts).unwrap();
        assert_eq!(key_orig.key.xprv, key_restored.key.xprv);

        let key_name_restored = "test_restore_fail".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::HexSeed,
            value: "X".to_string(),
            qr_version: 14,
        };
        let result = crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts);
        assert!(result.is_err());

        let key_name_restored = "test_restore_fail".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Xprv,
            value: key_orig.key.xpub.to_string(),
            qr_version: 14,
        };
        let result = crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts);
        assert!(result.is_err());

        let key_name_restored = "test_restore_fail".to_string();
        let restore_opts = RestoreOptions {
            key_name: key_name_restored,
            nature: Nature::Bech32Seed,
            value: "bc1q5lx5j4vedq9vj8rjm577annwxrppfda9hexah6".to_string(),
            qr_version: 14,
        };
        let result = crate::offline::restore::start(&temp_dir_str, Network::Testnet, &restore_opts);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "in network testnet bech32 seed must start with 'ts'"
        );
    }
}
