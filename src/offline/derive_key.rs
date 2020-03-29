use crate::sign::read_key;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::ChildNumber;
use bitcoin::Network;
use firma::{err, save_keys, PrivateMasterKey, MasterKeyOutput};
use std::path::PathBuf;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug)]
#[structopt(name = "restore")]
pub struct DeriveKeyOptions {
    /// Name of the master^2 key
    #[structopt(short, long)]
    from_key_file: PathBuf,

    /// Name of the generated master key, used as path to generate the child key
    #[structopt(short, long)]
    to_key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &DeriveKeyOptions) -> crate::Result<MasterKeyOutput> {
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
