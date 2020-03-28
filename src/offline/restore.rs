use crate::Result;
use bitcoin::bech32::{self, FromBase32};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use firma::{err, PrivateMasterKey, save_keys};
use serde_json::{Value, to_value};
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

pub fn start(datadir: &str, network: Network, opt: &RestoreOptions) -> Result<Value> {
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
        },
        Nature::HexSeed => {
            let sec = hex::decode(&opt.value)?;
            PrivateMasterKey::new(network, &sec)?
        },
    };

    let output = save_keys(datadir, network, &opt.key_name, master_key)?;

    Ok(to_value(&output)?)
}
