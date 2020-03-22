use crate::FirmaOnlineSubcommands::*;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::Network;
use bitcoincore_rpc::json::*;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use firma::*;
use log::{debug, info};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod balance;
mod create_tx;
mod create_wallet;
mod get_address;
mod list_coins;
mod rescan;
mod send_tx;

/// firma-online is an helper tool to use with bitcoin core, it allows to:
/// create a watch-only multisig wallet,
/// create a funded PSBT tx without signatures and
/// combine PSBT to create and broadcast a full tx
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-online")]
struct FirmaOnlineCommands {
    /// Verbose mode (-v)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(flatten)]
    context: Context,

    #[structopt(subcommand)]
    subcommand: FirmaOnlineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOnlineSubcommands {
    /// Create a new watch-only wallet
    CreateWallet(crate::create_wallet::CreateWalletOptions),

    /// Rescan the blockchain, useful when importing an existing wallet
    Rescan(crate::rescan::RescanOptions),

    /// Get a new address for given wallet
    GetAddress(crate::get_address::GetAddressOptions),

    /// Create a new transaction as unsigned PSBT
    CreateTx(crate::create_tx::CreateTxOptions),

    /// Combine signed PSBT from offline signers and send the resulting tx
    SendTx(crate::send_tx::SendTxOptions),

    /// View wallet balance
    Balance,

    /// View wallet coins
    ListCoins,
}

struct Wallet {
    client: Client,
    context: Context,
}

impl Wallet {
    pub fn new(url: String, auth: Auth, context: Context) -> Result<Self> {
        Ok(Wallet {
            client: Client::new(url, auth)?,
            context,
        })
    }
}

fn main() -> Result<()> {
    let output = match start() {
        Ok(output) => output,
        Err(e) => e.to_json()?,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn start() -> Result<Value> {
    let cmd = FirmaOnlineCommands::from_args();
    init_logger(cmd.verbose);
    debug!("{:?}", cmd);

    let daemon_opts = match &cmd.subcommand {
        CreateWallet(ref opt) => opt.daemon_opts.clone(),
        _ => {
            let (wallet, _) = cmd.context.load_wallet_and_index()?;
            wallet.daemon_opts
        }
    };

    let url_with_wallet = format!("{}/wallet/{}", daemon_opts.url, cmd.context.wallet_name);
    let wallet = Wallet::new(
        url_with_wallet,
        Auth::CookieFile(daemon_opts.cookie_file.clone()),
        cmd.context.clone(),
    )?;
    if !matches!(cmd.subcommand, CreateWallet(_)) {
        wallet.load_if_unloaded(&cmd.context.wallet_name)?;
    }

    let result = wallet.client.get_blockchain_info()?;
    debug!("{:?}", result);

    let node_network = match result.chain.as_ref() {
        "main" => Network::Bitcoin,
        "test" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => return err("Unrecognized network"),
    };
    if node_network != cmd.context.network {
        return err(&format!(
            "network of the bitcoin node {} does not match used one {}",
            node_network, cmd.context.network
        ));
    }

    let output = match cmd.subcommand {
        CreateWallet(ref opt) => wallet.create(&daemon_opts, opt)?,
        GetAddress(ref opt) => wallet.get_address_value(opt.index, false)?,
        CreateTx(ref opt) => wallet.create_tx(opt)?,
        SendTx(ref opt) => wallet.send_tx(opt)?,
        Balance => wallet.balance()?,
        Rescan(ref opt) => wallet.rescan(opt)?,
        ListCoins => wallet.list_coins()?,
    };

    Ok(output)
}

fn save_psbt(psbt: &WalletCreateFundedPsbtResult, datadir: &str) -> Result<PathBuf> {
    let mut count = 0;
    loop {
        let slash = if datadir.ends_with('/') { "" } else { "/" };
        let path = expand_tilde(format!("{}{}psbt-{}.json", datadir, slash, count))?;
        if !path.exists() {
            info!("Saving psbt in {:?}", path);
            fs::write(&path, serde_json::to_string_pretty(psbt)?)?;
            return Ok(path);
        }
        count += 1;
    }
}

fn read_xpubs_files(paths: &[PathBuf]) -> Result<Vec<ExtendedPubKey>> {
    let mut xpubs = vec![];
    for xpub_path in paths.iter() {
        let content = fs::read(xpub_path)?;
        let json: PublicMasterKey = serde_json::from_slice(&content)?;
        xpubs.push(json.xpub.clone());
    }
    Ok(xpubs)
}

impl Wallet {
    fn load_if_unloaded(&self, wallet_name: &str) -> Result<()> {
        match self.client.load_wallet(wallet_name) {
            Ok(_) => info!("wallet {} loaded", wallet_name),
            Err(e) => {
                if e.to_string().contains("not found") {
                    return err(&format!("{} not found in the bitcoin node", wallet_name));
                } else {
                    debug!("wallet {} already loaded", wallet_name);
                }
            }
        }
        Ok(())
    }
}
