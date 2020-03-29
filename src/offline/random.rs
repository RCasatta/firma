use bitcoin::Network;
use firma::common::save_keys;
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
    let sec = rand::thread_rng().gen::<[u8; 16]>();
    let master_key = PrivateMasterKey::new(network, &sec)?;
    let output = save_keys(datadir, network, &opt.key_name, master_key)?;

    Ok(to_value(&output)?)
}
