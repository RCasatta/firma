use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use firma::*;
use rand::Rng;
use std::error::Error;
use structopt::StructOpt;
use log::info;

type Result<R> = std::result::Result<R, Box<dyn Error>>;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &RandomOptions) -> Result<()> {
    let (private_file, public_file) = generate_key_filenames(datadir, network, &opt.key_name)?;

    let secp = Secp256k1::signing_only();
    let sec = rand::thread_rng().gen::<[u8; 32]>();
    let xpriv = ExtendedPrivKey::new_master(network, &sec)?;
    let xpub = ExtendedPubKey::from_private(&secp, &xpriv);

    let master_key = PrivateMasterKeyJson {
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
        faces: None,
        launches: None,
    };

    info!("{}", serde_json::to_string_pretty(&master_key)?);
    save_private(&master_key, &private_file);
    save_public(&master_key.into(), &public_file);

    Ok(())
}
