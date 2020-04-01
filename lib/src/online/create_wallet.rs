use crate::online::{read_xpubs_files, Wallet};
use crate::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::Network;
use bitcoincore_rpc::bitcoincore_rpc_json::{ImportMultiOptions, ImportMultiRequest};
use bitcoincore_rpc::RpcApi;
use log::debug;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CreateWalletOptions {
    /// number of signatures required
    #[structopt(short)]
    pub r: usize,

    /// Extended Public Keys (xpub) that are composing the wallet, given as String (xprv...)
    #[structopt(long = "xpub")]
    pub xpubs: Vec<ExtendedPubKey>,

    /// Extended Public Keys (xpub) that are composing the wallet, given as a json file
    #[structopt(long = "xpub-file")]
    pub xpub_files: Vec<PathBuf>,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,
}

impl CreateWalletOptions {
    fn validate(&self, network: Network) -> Result<()> {
        if self.r == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.r > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.r > (self.xpubs.len() + self.xpub_files.len()) {
            return Err("required signatures cannot be greater than the number of xpubs".into());
        }

        let mut xpubs = read_xpubs_files(&self.xpub_files)?;
        xpubs.extend(&self.xpubs);

        for xpub in xpubs.iter() {
            if !(network == xpub.network
                || (network == Network::Regtest && xpub.network == Network::Testnet))
            {
                return Err(format!(
                    "detected xpub of another network (cmd:{}) (xpub:{})",
                    network, xpub.network
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

impl Wallet {
    pub fn create(
        &self,
        daemon_opts: &DaemonOpts,
        opt: &CreateWalletOptions,
        height: u64,
    ) -> Result<CreateWalletOutput> {
        opt.validate(self.context.network)?;
        debug!("create");

        let mut xpubs = read_xpubs_files(&opt.xpub_files)?;
        xpubs.extend(&opt.xpubs);

        let mut descriptors = vec![];
        for i in 0..=1 {
            let mut xpub_paths = vec![];
            for xpub in xpubs.iter() {
                let xpub_path = format!("{}/{}/*", xpub, i);
                xpub_paths.push(xpub_path)
            }
            let descriptor = format!("wsh(multi({},{}))", opt.r, xpub_paths.join(","));
            descriptors.push(descriptor);
        }

        let descriptor_main = self.client.get_descriptor_info(&descriptors[0])?.descriptor;
        let descriptor_change = self.client.get_descriptor_info(&descriptors[1])?.descriptor;

        self.client
            .create_wallet(&self.context.wallet_name, Some(true))?;

        let mut multi_request: ImportMultiRequest = Default::default();
        multi_request.range = Some((0, 1000)); //TODO should be a parameter
        multi_request.timestamp = 0; //TODO init to current timestamp
        multi_request.keypool = Some(true);
        multi_request.watchonly = Some(true);
        let mut main = multi_request.clone();
        main.descriptor = Some(&descriptor_main);
        main.internal = Some(false);
        let mut change = multi_request.clone();
        change.descriptor = Some(&descriptor_change);
        change.internal = Some(true);

        let multi_options = ImportMultiOptions {
            rescan: Some(false),
        };

        let import_multi_result = self
            .client
            .import_multi(&[main, change], Some(&multi_options));
        info!("import_multi_result {:?}", import_multi_result);

        let fingerprints = xpubs.iter().map(|x| x.fingerprint()).collect();

        let wallet = WalletJson {
            name: self.context.wallet_name.to_string(),
            descriptor_main,
            descriptor_change,
            daemon_opts: Some(daemon_opts.clone()),
            fingerprints,
            required_sig: opt.r,
            created_at_height: height,
        };
        let indexes = WalletIndexes {
            main: 0u32,
            change: 0u32,
        };

        let wallet_file = self.context.save_wallet(&wallet)?;
        self.context.save_index(&indexes)?;

        let mut wallet_for_qr = wallet.clone();
        wallet_for_qr.daemon_opts = None; // no need of this info in the qr code
        let qr_bytes = serde_json::to_vec(&wallet_for_qr)?;

        let wallet_qr_path = self.context.path_for_wallet_qr()?;
        let qr_files = common::qr::save_qrs(qr_bytes, wallet_qr_path, opt.qr_version)?;

        let create_wallet = CreateWalletOutput {
            qr_files,
            wallet_file,
            wallet,
        };

        Ok(create_wallet)
    }
}
