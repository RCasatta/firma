use crate::FirmaOnlineSubcommands::*;
use bitcoin::Network;
use bitcoincore_rpc::json::*;
use bitcoincore_rpc::{Auth, RpcApi};
use firma::*;
use log::debug;
use serde_json::Value;
use std::convert::TryInto;
use structopt::StructOpt;

/// firma-online is an helper tool to use with bitcoin core, it allows to:
/// create a watch-only multisig wallet,
/// create a funded PSBT tx without signatures and
/// combine PSBT to create and broadcast a full tx
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-online")]
struct FirmaOnlineCommands {
    #[structopt(flatten)]
    context: Context,

    #[structopt(subcommand)]
    subcommand: FirmaOnlineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOnlineSubcommands {
    /// Create a new watch-only wallet
    CreateWallet(firma::online::create_wallet::CreateWalletOptions),

    /// Rescan the blockchain, useful when importing an existing wallet
    Rescan(firma::online::rescan::RescanOptions),

    /// Get a new address for given wallet
    GetAddress(firma::online::get_address::GetAddressOptions),

    /// Create a new transaction as unsigned PSBT
    CreateTx(firma::online::create_tx::CreateTxOptions),

    /// Combine signed PSBT from offline signers and send the resulting tx
    SendTx(firma::online::send_tx::SendTxOptions),

    /// View wallet balance
    Balance,

    /// View wallet coins
    ListCoins,
}

fn main() -> Result<()> {
    let output = match start() {
        Ok(output) => output,
        Err(e) => e.to_json(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn start() -> Result<Value> {
    init_logger();
    debug!("firma-online start");
    let cmd = FirmaOnlineCommands::from_args();

    let daemon_opts = match &cmd.subcommand {
        CreateWallet(ref opt) => opt.daemon_opts.clone(),
        _ => {
            let (wallet, _) = cmd.context.load_wallet_and_index()?;
            wallet
                .daemon_opts
                .ok_or_else(|| Error::Generic("daemon_opts missing".into()))?
        }
    };

    let url_with_wallet = format!("{}/wallet/{}", daemon_opts.url, cmd.context.wallet_name);
    let wallet = Wallet::new(
        url_with_wallet,
        Auth::CookieFile(daemon_opts.cookie_file.clone()),
        cmd.context.clone(),
    )?;

    if let CreateWallet(_) = cmd.subcommand {
        // do nothing, I need the else branch (!matches!() require too recent rust version)
    } else {
        wallet.load_if_unloaded(&cmd.context.wallet_name)?;
    }

    let result = wallet.client.get_blockchain_info()?;

    let node_network = match result.chain.as_ref() {
        "main" => Network::Bitcoin,
        "test" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => return Err("Unrecognized network".into()),
    };
    if node_network != cmd.context.network {
        return Err(format!(
            "network of the bitcoin node {} does not match used one {}",
            node_network, cmd.context.network
        )
        .into());
    }

    match cmd.subcommand {
        CreateWallet(ref opt) => wallet.create(&daemon_opts, opt, result.blocks)?.try_into(),
        GetAddress(ref opt) => wallet.get_address(opt.index, false)?.try_into(),
        CreateTx(ref opt) => wallet.create_tx(opt)?.try_into(),
        SendTx(ref opt) => wallet.send_tx(opt)?.try_into(),
        Balance => wallet.balance()?.try_into(),
        Rescan(ref opt) => Ok(wallet.rescan(opt)?),
        ListCoins => wallet.list_coins()?.try_into(),
    }
}
