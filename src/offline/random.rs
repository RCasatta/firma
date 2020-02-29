use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use firma::{name_to_path, save, MasterKeyJson};
use log::info;
use rand::Rng;
use std::error::Error;
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &RandomOptions) -> Result<(), Box<dyn Error>> {
    let output = name_to_path(datadir, &opt.key_name, "key.json");
    if output.exists() {
        return Err(format!(
            "Output file {:?} exists, exiting to avoid unwanted override. Run --help.",
            &output
        )
        .into());
    }
    let secp = Secp256k1::signing_only();
    let sec = rand::thread_rng().gen::<[u8; 32]>();
    let xpriv = ExtendedPrivKey::new_master(network, &sec)?;
    let xpub = ExtendedPubKey::from_private(&secp, &xpriv);

    let master_key = MasterKeyJson {
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
        faces: None,
        launches: None,
    };

    let filename = save(&master_key, &output);
    info!("key saved in {}", filename);

    Ok(())
}
