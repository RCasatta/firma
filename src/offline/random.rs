use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use firma::*;
use rand::Rng;
use serde_json::{to_value, Value};
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &RandomOptions) -> Result<Value> {
    let (private_key_file, public_key_file) =
        generate_key_filenames(datadir, network, &opt.key_name)?;
    let secp = Secp256k1::signing_only();
    let sec = rand::thread_rng().gen::<[u8; 32]>();
    let xprv = ExtendedPrivKey::new_master(network, &sec)?;
    let xpub = ExtendedPubKey::from_private(&secp, &xprv);

    let key = PrivateMasterKey {
        xprv,
        xpub,
        faces: None,
        launches: None,
    };

    save_private(&key, &private_key_file)?;
    save_public(&key.clone().into(), &public_key_file)?;

    let output = MasterKeyOutput {
        key,
        public_file: public_key_file,
        private_file: private_key_file,
    };
    Ok(to_value(&output)?)
}
