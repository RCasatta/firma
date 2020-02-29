use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::{Address, Amount, Network};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::json::{ImportMultiOptions, ImportMultiRequest, WalletCreateFundedPsbtOptions};
use firma::{init_logger, name_to_path, DaemonOpts, WalletIndexes, WalletJson};
use log::{debug, info};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use structopt::StructOpt;

/// firma-online is an helper tool to use with bitcoin core, it allows to:
/// * Create a watch-only multisig wallet
/// * Create a funded PSBT tx without signatures
/// * Combine PSBT to create and broadcast a full tx
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-online")]
struct FirmaOnlineCommands {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// Name of the wallet
    #[structopt(short, long)]
    wallet_name: String,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    firma_datadir: String,

    #[structopt(flatten)]
    daemon_opts: DaemonOpts,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    subcommand: FirmaOnlineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOnlineSubcommands {
    CreateWallet(CreateWalletOptions),
    Rescan(RescanOptions),
    GetAddress(GetAddressOptions),
    CreateTx(CreateTxOptions),
    SendTx(SendTxOptions),
    Balance,
}

#[derive(StructOpt, Debug)]
pub struct CreateWalletOptions {
    /// number of signatures required
    #[structopt(short)]
    r: usize,

    /// Extended Public Keys (xpub) that are composing the wallet
    #[structopt(short, long = "xpub")]
    xpubs: Vec<ExtendedPubKey>,
}

#[derive(StructOpt, Debug)]
pub struct GetAddressOptions {
    #[structopt(long)]
    index: Option<u32>,
}

#[derive(StructOpt, Debug)]
pub struct RescanOptions {
    #[structopt(long)]
    start_from: Option<usize>,
}

#[derive(StructOpt, Debug)]
pub struct CreateTxOptions {
    #[structopt(long)]
    address: Address,
    #[structopt(long)]
    amount: Amount,
}

#[derive(StructOpt, Debug)]
pub struct SendTxOptions {
    #[structopt(long = "psbt")]
    psbts: Vec<String>,

    #[structopt(long)]
    dry_run: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = FirmaOnlineCommands::from_args();

    init_logger(cmd.verbose);

    debug!("{:?}", cmd);

    let url_with_wallet = format!("{}/wallet/{}", cmd.daemon_opts.url, cmd.wallet_name);
    let client = Client::new(
        url_with_wallet,
        Auth::UserPass(
            cmd.daemon_opts.rpcuser.clone(),
            cmd.daemon_opts.rpcpassword.clone(),
        ),
    )?;
    let result = client.get_blockchain_info();

    debug!("{:?}", result);

    match result.unwrap().chain.as_ref() {
        "main" => assert_eq!(Network::Bitcoin, cmd.network),
        "test" => assert_eq!(Network::Testnet, cmd.network),
        "regtest" => assert_eq!(Network::Regtest, cmd.network),
        _ => return Err("Unrecognized network".into()),
    };

    match cmd.subcommand {
        FirmaOnlineSubcommands::CreateWallet(opt) => create_wallet(
            &client,
            &cmd.network,
            &cmd.firma_datadir,
            &cmd.wallet_name,
            &cmd.daemon_opts,
            &opt,
        )?,
        FirmaOnlineSubcommands::GetAddress(opt) => get_address(
            &client,
            &cmd.network,
            &cmd.firma_datadir,
            &cmd.wallet_name,
            &opt.index,
            false,
        )
        .map(|_| ())?,
        FirmaOnlineSubcommands::CreateTx(opt) => create_tx(
            &client,
            &cmd.network,
            &cmd.firma_datadir,
            &cmd.wallet_name,
            &opt,
        )?,
        FirmaOnlineSubcommands::SendTx(opt) => send_tx(&client, &opt)?,
        FirmaOnlineSubcommands::Balance => balance(&client)?,
        FirmaOnlineSubcommands::Rescan(opt) => rescan(&client, &opt)?,
    }

    Ok(())
}

fn get_address(
    client: &Client,
    network: &Network,
    datadir: &str,
    wallet_name: &str,
    cmd_index: &Option<u32>,
    is_change: bool,
) -> Result<Address, Box<dyn Error>> {
    let (wallet, mut index_json) = load_wallet_and_index(datadir, wallet_name)?;

    let (index, descriptor) = if is_change {
        (index_json.change, wallet.change_descriptor)
    } else {
        match cmd_index {
            Some(index) => (index.clone(), wallet.main_descriptor),
            None => (index_json.main, wallet.main_descriptor),
        }
    };

    let addresses = client.derive_addresses(&descriptor, [index, index])?;
    let address = &addresses[0];
    if &address.network != network {
        return Err("address returned is not on the same network as given".into());
    }
    info!("{}", address);

    if is_change {
        index_json.change += 1;
        save_index(&index_json, datadir, wallet_name)?;
    } else {
        if cmd_index.is_none() {
            index_json.main += 1;
            save_index(&index_json, datadir, wallet_name)?;
        }
    }

    Ok(address.clone())
}

fn create_wallet(
    client: &Client,
    network: &Network,
    datadir: &str,
    wallet_name: &str,
    daemon_opts: &DaemonOpts,
    opt: &CreateWalletOptions,
) -> Result<(), Box<dyn Error>> {
    opt.validate(&network)?;

    let mut descriptors = vec![];
    for i in 0..=1 {
        let mut xpub_paths = vec![];
        for xpub in opt.xpubs.iter() {
            let xpub_path = format!("{}/{}/*", xpub, i);
            xpub_paths.push(xpub_path)
        }
        let descriptor = format!("wsh(multi({},{}))", opt.r, xpub_paths.join(","));
        descriptors.push(descriptor);
    }
    dbg!(&descriptors);

    let main_descriptor = client.get_descriptor_info(&descriptors[0])?.descriptor;
    let change_descriptor = client.get_descriptor_info(&descriptors[1])?.descriptor;
    dbg!(&main_descriptor);
    dbg!(&change_descriptor);

    client.create_wallet(wallet_name, Some(true))?;

    let mut multi_request: ImportMultiRequest = Default::default();
    multi_request.range = Some((0, 1000)); //TODO should be a parameter
    multi_request.timestamp = 0; //TODO init to current timestamp
    multi_request.keypool = Some(true);
    multi_request.watchonly = Some(true);
    let mut main = multi_request.clone();
    main.descriptor = Some(&main_descriptor);
    main.internal = Some(false);
    let mut change = multi_request.clone();
    change.descriptor = Some(&change_descriptor);
    change.internal = Some(true);

    let multi_options = ImportMultiOptions {
        rescan: Some(false),
    };

    let import_multi_result = client.import_multi(&[main, change], Some(&multi_options));
    info!("import_multi_result {:?}", import_multi_result);
    //import_multi_result Ok([ImportMultiResult { success: true, warnings: [], error: None }, ImportMultiResult { success: true, warnings: [], error: None }])

    let wallet = WalletJson {
        name: wallet_name.to_string(),
        main_descriptor,
        change_descriptor,
        daemon_opts: daemon_opts.clone(),
    };
    let indexes = WalletIndexes {
        main: 0u32,
        change: 0u32,
    };

    save_wallet(&wallet, datadir, wallet_name)?;
    save_index(&indexes, datadir, wallet_name)?;

    Ok(())
}

impl CreateWalletOptions {
    fn validate(&self, network: &Network) -> Result<(), Box<dyn Error>> {
        if self.r == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.r > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.r > self.xpubs.len() {
            return Err("required signatures cannot be greater than the number of xpubs".into());
        }

        for xpub in self.xpubs.iter() {
            if network != &xpub.network {
                return Err("detected xpub of another network".into());
            }

            if self.xpubs.iter().filter(|xpub2| *xpub2 == xpub).count() > 1 {
                return Err("Cannot use same xpub twice".into());
            }
        }

        Ok(())
    }
}

fn create_tx(
    client: &Client,
    network: &Network,
    datadir: &str,
    wallet_name: &str,
    opt: &CreateTxOptions,
) -> Result<(), Box<dyn Error>> {
    //bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} walletcreatefundedpsbt
    // '[]' '[{"tb1qrxye2d9e5qgsg0qd647rl7drs8p4ytzlylr2ggceppd4djj58gws84d0gv":0.0012345}]'
    // 0 '{"includeWatching":true, "changeAddress":"tb1qmkzvhdr23alghczwyaj0p2zxvs73ysxene09c53yl0ven2xfwc5q82artm"}' true> psbt_2.txt

    let mut outputs = HashMap::new();
    outputs.insert(opt.address.to_string(), opt.amount);
    debug!("{:?}", outputs);

    let mut options: WalletCreateFundedPsbtOptions = Default::default();
    options.include_watching = Some(true);
    options.change_address = Some(get_address(
        client,
        network,
        datadir,
        wallet_name,
        &None,
        true,
    )?);
    let b = client.wallet_create_funded_psbt(&[], &outputs, None, Some(options), Some(true));
    debug!("wallet_create_funded_psbt {:?}", b);

    Ok(())
}

fn balance(client: &Client) -> Result<(), Box<dyn Error>> {
    let balance = client.get_balance(Some(0), Some(true))?;
    info!("{}", balance);
    Ok(())
}

fn send_tx(client: &Client, opt: &SendTxOptions) -> Result<(), Box<dyn Error>> {
    let combined = client.combine_psbt(&opt.psbts)?;
    debug!("combined {:?}", combined);

    let finalized = client.finalize_psbt(&combined, true)?;
    debug!("finalized {:?}", finalized);

    if !opt.dry_run {
        let hash = client.send_raw_transaction(finalized.hex.unwrap())?;
        info!("txid {:?}", hash);
    }

    Ok(())
}

fn rescan(client: &Client, opt: &RescanOptions) -> Result<(), Box<dyn Error>> {
    client.rescan_blockchain(opt.start_from)?;
    Ok(())
}

fn load_wallet_and_index(
    datadir: &str,
    wallet_name: &str,
) -> Result<(WalletJson, WalletIndexes), Box<dyn Error>> {
    let wallet_path = name_to_path(datadir, wallet_name, "descriptor.json");
    let wallet = fs::read(wallet_path)?;
    let wallet = serde_json::from_slice(&wallet)?;

    let indexes_path = name_to_path(datadir, wallet_name, "indexes.json");
    let indexes = fs::read(indexes_path)?;
    let indexes = serde_json::from_slice(&indexes)?;

    Ok((wallet, indexes))
}

fn save_wallet(
    wallet: &WalletJson,
    datadir: &str,
    wallet_name: &str,
) -> Result<(), Box<dyn Error>> {
    let path = name_to_path(datadir, wallet_name, "descriptor.json");
    if path.exists() {
        return Err("wallet already exist, I am not going to overwrite".into());
    }
    info!("Saving wallet data in {:?}", path);

    fs::write(path, serde_json::to_string_pretty(wallet)?)?;
    Ok(())
}

fn save_index(
    indexes: &WalletIndexes,
    datadir: &str,
    wallet_name: &str,
) -> Result<(), Box<dyn Error>> {
    let path = name_to_path(datadir, wallet_name, "indexes.json");
    fs::write(path, serde_json::to_string_pretty(indexes)?)?;
    Ok(())
}
