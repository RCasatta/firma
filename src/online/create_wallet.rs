use crate::*;
use bitcoin::Network;
use bitcoincore_rpc::RpcApi;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CreateWalletOptions {
    /// number of signatures required
    #[structopt(short)]
    pub r: usize,

    /// Extended Public Keys (xpub) that are composing the wallet
    #[structopt(short, long = "xpub")]
    pub xpubs: Vec<PathBuf>,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,
}

impl CreateWalletOptions {
    fn validate(&self, network: Network) -> Result<()> {
        if self.r == 0 {
            return err("required signatures cannot be 0");
        }

        if self.r > 15 {
            return err("required signatures cannot be greater than 15");
        }

        if self.r > self.xpubs.len() {
            return err("required signatures cannot be greater than the number of xpubs");
        }

        let xpubs = read_xpubs_files(&self.xpubs)?;
        for xpub in xpubs.iter() {
            if !(network == xpub.network
                || (network == Network::Regtest && xpub.network == Network::Testnet))
            {
                return err(&format!(
                    "detected xpub of another network (cmd:{}) (xpub:{})",
                    network, xpub.network
                ));
            }

            if xpubs.iter().filter(|xpub2| *xpub2 == xpub).count() > 1 {
                return err("Cannot use same xpub twice");
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
    ) -> Result<CreateWalletOutput> {
        opt.validate(self.context.network)?;

        let xpubs = read_xpubs_files(&opt.xpubs)?;

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
        //dbg!(&descriptors);

        let descriptor_main = self.client.get_descriptor_info(&descriptors[0])?.descriptor;
        let descriptor_change = self.client.get_descriptor_info(&descriptors[1])?.descriptor;
        //dbg!(&main_descriptor);
        //dbg!(&change_descriptor);

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
        let kind = format!("{:02}of{:02}", opt.r, opt.xpubs.len());
        let position = LABELS.iter().position(|el| el == &&kind).unwrap_or(10); // 10 is the other class
        let total: f64 = VALUES.iter().sum::<u64>() as f64;
        let percentage: Vec<f64> = VALUES.iter().map(|v| (*v as f64) / total).collect();

        let diffusion = match position {
            10 => format!("less than {:.6}", percentage[position]),
            _ => format!("{:.6}", percentage[position]),
        };
        let stat = KindStat { kind, diffusion };
        let fingerprints = xpubs.iter().map(|x| x.fingerprint()).collect();

        let wallet = WalletJson {
            name: self.context.wallet_name.to_string(),
            descriptor_main,
            descriptor_change,
            daemon_opts: daemon_opts.clone(),
            stat,
            fingerprints,
        };
        let indexes = WalletIndexes {
            main: 0u32,
            change: 0u32,
        };

        let wallet_file = self.context.save_wallet(&wallet)?;
        self.context.save_index(&indexes)?;

        let create_wallet = CreateWalletOutput {
            wallet_file,
            wallet,
        };

        Ok(create_wallet)
    }
}

// From https://opreturn.org/segwit-multisig/ # 1 march 2020
const LABELS: [&str; 11] = [
    "01of01", "01of02", "01of04", "02of02", "02of03", "02of04", "02of05", "03of04", "03of05",
    "04of05", "other",
];
const VALUES: [u64; 11] = [
    2770896, 128768, 22264, 16476433, 23682698, 286534, 1783, 67716, 2760, 38641, 1912,
];
