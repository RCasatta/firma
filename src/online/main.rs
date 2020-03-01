use crate::FirmaOnlineSubcommands::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::{Address, Amount, Network};
use bitcoincore_rpc::bitcoincore_rpc_json::WalletCreateFundedPsbtResult;
use bitcoincore_rpc::json::*;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use firma::*;
use log::{debug, info};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

type Result<R> = std::result::Result<R, Box<dyn Error>>;

/// firma-online is an helper tool to use with bitcoin core, it allows to:
/// * Create a watch-only multisig wallet
/// * Create a funded PSBT tx without signatures
/// * Combine PSBT to create and broadcast a full tx
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-online")]
struct FirmaOnlineCommands {
    /// Verbose mode (-v)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(flatten)]
    context: Context,

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
    xpubs: Vec<PathBuf>,

    #[structopt(flatten)]
    daemon_opts: DaemonOpts,
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

// TODO consider using Vec<BitcoinUri> with address and amount in a single string to support sending to multiple recipient
#[derive(StructOpt, Debug)]
pub struct CreateTxOptions {
    /// address of the recipient
    #[structopt(long)]
    address: Address,

    /// amount with unit eg "5000 sat" or "1.1 btc", use quotes.
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

fn main() -> Result<()> {
    let cmd = FirmaOnlineCommands::from_args();
    init_logger(cmd.verbose);
    debug!("{:?}", cmd);
    let context = &cmd.context;

    let daemon_opts = match &cmd.subcommand {
        CreateWallet(ref opt) => opt.daemon_opts.clone(),
        _ => {
            let (wallet, _) = cmd.context.load_wallet_and_index()?;
            wallet.daemon_opts.clone()
        },
    };

    let url_with_wallet = format!("{}/wallet/{}", daemon_opts.url, context.wallet_name);
    let client = Client::new(
        url_with_wallet,
        Auth::CookieFile(daemon_opts.cookie_file.clone()),
    )?;
    let result = client.get_blockchain_info()?;

    debug!("{:?}", result);

    match result.chain.as_ref() {
        "main" => assert_eq!(Network::Bitcoin, context.network),
        "test" => assert_eq!(Network::Testnet, context.network),
        "regtest" => assert_eq!(Network::Regtest, context.network),
        _ => return Err("Unrecognized network".into()),
    };

    match cmd.subcommand {
        CreateWallet(ref opt) => create_wallet(&client, &cmd.context, &daemon_opts, opt)?,
        GetAddress(ref opt) => get_address(&client, &cmd.context, &opt.index, false).map(|_| ())?,
        CreateTx(ref opt) => create_tx(&client, &cmd.context, opt)?,
        SendTx(opt) => send_tx(&client, &opt)?,
        Balance => balance(&client)?,
        Rescan(opt) => rescan(&client, &opt)?,
    }

    Ok(())
}

fn get_address(
    client: &Client,
    context: &Context,
    cmd_index: &Option<u32>,
    is_change: bool,
) -> Result<Address> {
    let (wallet, mut index_json) = context.load_wallet_and_index()?;

    let (index, descriptor) = if is_change {
        (index_json.change, wallet.change_descriptor)
    } else {
        match cmd_index {
            Some(index) => (index.clone(), wallet.main_descriptor),
            None => (index_json.main, wallet.main_descriptor),
        }
    };
    let address_type = match is_change {
        true => "change",
        false => "external",
    };

    info!("Creating {} address at index {}", address_type, index);
    let addresses = client.derive_addresses(&descriptor, [index, index])?;
    let address = &addresses[0];
    if address.network != context.network {
        return Err("address returned is not on the same network as given".into());
    }
    info!("{}", address);

    if is_change {
        index_json.change += 1;
        context.save_index(&index_json)?;
    } else {
        if cmd_index.is_none() {
            index_json.main += 1;
            context.save_index(&index_json)?;
        }
    }

    Ok(address.clone())
}

fn create_wallet(
    client: &Client,
    context: &Context,
    daemon_opts: &DaemonOpts,
    opt: &CreateWalletOptions,
) -> Result<()> {
    opt.validate(&context.network)?;

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
    dbg!(&descriptors);

    let main_descriptor = client.get_descriptor_info(&descriptors[0])?.descriptor;
    let change_descriptor = client.get_descriptor_info(&descriptors[1])?.descriptor;
    dbg!(&main_descriptor);
    dbg!(&change_descriptor);

    client.create_wallet(&context.wallet_name, Some(true))?;

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
        name: context.wallet_name.to_string(),
        main_descriptor,
        change_descriptor,
        daemon_opts: daemon_opts.clone(),
    };
    let indexes = WalletIndexesJson {
        main: 0u32,
        change: 0u32,
    };

    context.save_wallet(&wallet)?;
    context.save_index(&indexes)?;

    Ok(())
}

impl CreateWalletOptions {
    fn validate(&self, network: &Network) -> Result<()> {
        if self.r == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.r > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.r > self.xpubs.len() {
            return Err("required signatures cannot be greater than the number of xpubs".into());
        }

        let xpubs = read_xpubs_files(&self.xpubs)?;
        for xpub in xpubs.iter() {
            if network != &xpub.network {
                return Err("detected xpub of another network".into());
            }

            if xpubs.iter().filter(|xpub2| *xpub2 == xpub).count() > 1 {
                return Err("Cannot use same xpub twice".into());
            }
        }

        Ok(())
    }
}

fn create_tx(client: &Client, context: &Context, opt: &CreateTxOptions) -> Result<()> {
    let mut outputs = HashMap::new();
    outputs.insert(opt.address.to_string(), opt.amount);
    debug!("{:?}", outputs);

    let mut options: WalletCreateFundedPsbtOptions = Default::default();
    options.include_watching = Some(true);
    options.change_address = Some(get_address(client, context, &None, true)?);
    let b = client.wallet_create_funded_psbt(&[], &outputs, None, Some(options), Some(true))?;
    info!("wallet_create_funded_psbt {:#?}", b);

    save_psbt(b)?;

    Ok(())
}

fn balance(client: &Client) -> Result<()> {
    let balance = client.get_balance(Some(0), Some(true))?;
    info!("{}", balance);
    Ok(())
}

fn send_tx(client: &Client, opt: &SendTxOptions) -> Result<()> {
    let mut psbts = vec![];
    for psbt_file in opt.psbts.iter() {
        let path = Path::new(psbt_file);
        let json = read_psbt(path.into());
        psbts.push(json.signed_psbt.expect("signed_psbt not found"));
    }
    let combined = client.combine_psbt(&psbts)?;
    debug!("combined {:?}", combined);

    let finalized = client.finalize_psbt(&combined, true)?;
    debug!("finalized {:?}", finalized);

    if !opt.dry_run {
        let hash = client.send_raw_transaction(finalized.hex.unwrap())?;
        info!("txid {:?}", hash);
    }

    Ok(())
}

fn rescan(client: &Client, opt: &RescanOptions) -> Result<()> {
    client.rescan_blockchain(opt.start_from)?;
    Ok(())
}

fn save_psbt(psbt: WalletCreateFundedPsbtResult) -> Result<()> {
    let mut count = 0;
    loop {
        let filename = format!("psbt-{}.json", count);
        let path = Path::new(&filename);
        if !path.exists() {
            info!("Saving psbt in {:?}", path);
            fs::write(path, serde_json::to_string_pretty(&psbt)?)?;
            return Ok(());
        }
        count += 1;
    }
}

fn read_xpubs_files(paths: &Vec<PathBuf>) -> Result<Vec<ExtendedPubKey>> {
    let mut xpubs = vec![];
    for xpub_path in paths.iter() {
        let content = fs::read(xpub_path)?;
        let json: PublicMasterKeyJson = serde_json::from_slice(&content)?;
        xpubs.push(ExtendedPubKey::from_str(&json.xpub)?);
    }
    Ok(xpubs)
}
