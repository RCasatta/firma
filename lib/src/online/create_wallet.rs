use crate::common::json::identifier::{Identifier, Kind};
use crate::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::Network;
use bitcoincore_rpc::bitcoincore_rpc_json::{
    ImportMultiOptions, ImportMultiRequest, ImportMultiRescanSince,
};
use bitcoincore_rpc::RpcApi;
use log::debug;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CreateWalletOptions {
    /// The name of the wallet to be created
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,

    /// number of signatures required
    #[structopt(short)]
    pub r: usize,

    /// Extended Public Keys (xpub) that are composing the wallet, given as String (xprv...)
    #[structopt(long = "xpub")]
    pub xpubs: Vec<ExtendedPubKey>, //TODO change in DescriptorPubKey

    /// Key name that are composing the wallet, must be found in firma datadir
    #[structopt(long = "key-name")]
    pub key_names: Vec<String>,
}

impl CreateWalletOptions {
    fn validate(&self, context: &Context) -> Result<()> {
        if self.r == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.r > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.r > (self.xpubs.len() + self.key_names.len()) {
            return Err("required signatures cannot be greater than the number of xpubs".into());
        }

        let mut xpubs = context.read_xpubs_from_names(&self.key_names)?;
        xpubs.extend(&self.xpubs);

        for xpub in xpubs.iter() {
            if !(context.network == xpub.network
                || (context.network == Network::Regtest && xpub.network == Network::Testnet))
            {
                return Err(format!(
                    "detected xpub of another network (cmd:{}) (xpub:{})",
                    context.network, xpub.network
                )
                .into());
            }

            if xpubs.iter().filter(|xpub2| *xpub2 == xpub).count() > 1 {
                return Err("Cannot use same xpub twice".into());
            }
        }

        Ok(())
    }
}

impl Context {
    pub fn create(&self, opt: &CreateWalletOptions) -> Result<WalletJson> {
        opt.validate(self)?;
        let client = self.make_client(&opt.wallet_name)?;
        debug!("create");

        let mut xpubs = self.read_xpubs_from_names(&opt.key_names)?;
        xpubs.extend(&opt.xpubs);

        let xpub_paths: Vec<String> = xpubs.iter().map(|xpub| format!("{}/0/*", xpub)).collect();
        let descriptor = format!("wsh(multi({},{}))", opt.r, xpub_paths.join(","));
        let descriptor = client.get_descriptor_info(&descriptor)?.descriptor; // adds checksum

        let multi_request = ImportMultiRequest {
            range: Some((0, 1000)), //TODO should be a parameter
            timestamp: ImportMultiRescanSince::Now,
            keypool: Some(true),
            watchonly: Some(true),
            descriptor: Some(&descriptor),
            internal: Some(false),
            ..Default::default()
        };

        let multi_options = ImportMultiOptions {
            rescan: Some(false),
        };

        let import_multi_result = client.import_multi(&[multi_request], Some(&multi_options));
        info!("import_multi_result {:?}", import_multi_result);

        let fingerprints = xpubs.iter().map(|x| x.fingerprint()).collect();
        let height = client.get_blockchain_info()?.blocks;

        let wallet = WalletJson {
            id: Identifier::new(self.network, Kind::Wallet, &opt.wallet_name),
            descriptor,
            fingerprints,
            required_sig: opt.r,
            created_at_height: height,
        };
        let indexes = IndexesJson {
            id: Identifier::new(self.network, Kind::WalletIndexes, &opt.wallet_name),
            main: 0u32,
        };

        self.write(&wallet)?;
        self.write(&indexes)?;

        Ok(wallet)
    }
}
