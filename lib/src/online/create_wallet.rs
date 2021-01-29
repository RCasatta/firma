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
    pub required_sigs: u8,

    /// Extended Public Keys (xpub) that are composing the wallet, given as String (xprv...)
    #[structopt(long = "xpub")]
    pub xpubs: Vec<ExtendedPubKey>, //TODO change in DescriptorPubKey

    /// Key name that are composing the wallet, must be found in firma datadir
    #[structopt(long = "key-name")]
    pub key_names: Vec<String>,

    /// If true, does not error if the wallet to be created already exists in the bitcoin node
    /// useful for testing locally, to avoid removing the wallets
    #[structopt(long)]
    pub allow_wallet_already_exists: bool,
}

impl CreateWalletOptions {
    fn validate(&self, context: &Context) -> Result<()> {
        if self.required_sigs == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.required_sigs > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.required_sigs > (self.xpubs.len() + self.key_names.len()) as u8 {
            //TODO check overflow
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
    pub fn create_wallet(&self, opt: &CreateWalletOptions) -> Result<WalletJson> {
        debug!("create_wallet {:?}", opt);
        opt.validate(self)?;

        // create the wallet in the bitcoin node  (should not already exist unless forced)
        match self.make_client(&opt.wallet_name) {
            Ok(_) => {
                if !opt.allow_wallet_already_exists {
                    return Err(Error::WalletAlreadyExistsInNode(
                        opt.wallet_name.to_string(),
                    ));
                }
            }
            Err(Error::WalletNotExistsInNode(_)) => {
                self.read_daemon_opts()?
                    .make_client(None, self.network)?
                    .create_wallet(&opt.wallet_name, Some(true), None, None, None)?;
            }
            Err(e) => return Err(e),
        };
        let client = self.make_client(&opt.wallet_name)?;

        let mut xpubs = self.read_xpubs_from_names(&opt.key_names)?;
        xpubs.extend(&opt.xpubs);

        let descriptor = create_descriptor(opt.required_sigs, &xpubs);
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
            required_sig: opt.required_sigs,
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

fn create_descriptor(required_sigs: u8, xpubs: &[ExtendedPubKey]) -> String {
    let xpub_paths: Vec<String> = xpubs.iter().map(|xpub| format!("{}/0/*", xpub)).collect();
    let descriptor = format!("wsh(multi({},{}))", required_sigs, xpub_paths.join(","));
    descriptor
}

#[cfg(test)]
mod tests {
    use crate::common::tests::rnd_string;
    use crate::online::create_wallet::{create_descriptor, CreateWalletOptions};
    use crate::{Identifier, Kind, MasterSecretJson, WalletJson};
    use bitcoin::Network;

    impl CreateWalletOptions {
        pub fn new_random(required_sigs: u8, key_names: Vec<String>) -> Self {
            CreateWalletOptions {
                wallet_name: rnd_string(),
                required_sigs,
                xpubs: vec![],
                key_names,
                allow_wallet_already_exists: false,
            }
        }
    }

    impl WalletJson {
        pub fn new_random(required_sig: u8, keys: &[MasterSecretJson]) -> Self {
            let xpubs: Vec<_> = keys.iter().map(|k| k.xpub.clone()).collect();
            Self {
                id: Identifier {
                    kind: Kind::Wallet,
                    name: rnd_string(),
                    network: Network::Testnet,
                },
                descriptor: create_descriptor(required_sig, &xpubs),
                fingerprints: Default::default(),
                required_sig,
                created_at_height: 0,
            }
        }
    }
}
