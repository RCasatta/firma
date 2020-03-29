use bitcoin::Network;
use firma::*;
use rand::Rng;
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    pub key_name: String,
}

pub fn start(datadir: &str, network: Network, opt: &RandomOptions) -> Result<MasterKeyOutput> {
    let sec = rand::thread_rng().gen::<[u8; 16]>();
    let master_key = PrivateMasterKey::new(network, &sec)?;
    let output = save_keys(datadir, network, &opt.key_name, master_key)?;

    Ok(output)
}
