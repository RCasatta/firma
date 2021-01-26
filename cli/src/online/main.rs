use firma::log::debug;
use firma::online::{ConnectOptions, PathOptions, WalletNameOptions};
use firma::serde_json::Value;
use firma::*;
use std::convert::TryInto;
use structopt::StructOpt;
use FirmaOnlineSubcommands::*;

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

    /// Flag to indicate usage of encryption/decryption when using CLI
    /// when true, reading from stdin is expected and blocking
    #[structopt(short, long)]
    encrypt: bool,
}

#[derive(StructOpt, Debug)]
enum FirmaOnlineSubcommands {
    /// Connect a bitcoin node
    Connect(ConnectOptions),

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
    Balance(WalletNameOptions),

    /// View wallet coins
    ListCoins(WalletNameOptions),

    /// Import the file containing a firma json object
    Import(PathOptions),
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
    let FirmaOnlineCommands {
        mut context,
        subcommand,
        encrypt,
    } = FirmaOnlineCommands::from_args();

    if encrypt {
        context.read_encryption_key()?;
    }

    debug!(
        "firma-online context:{:?} encrypt:{} subcommand:{:?}",
        context, encrypt, subcommand
    );

    match subcommand {
        Connect(opt) => {
            let _ = opt.daemon_opts.make_client(None, context.network)?;
            context.write_daemon_opts(opt.daemon_opts)?.try_into()
        }
        CreateWallet(opt) => context.create(&opt)?.try_into(),
        GetAddress(opt) => context.get_address(&opt)?.try_into(),
        CreateTx(opt) => context.create_tx(&opt)?.try_into(),
        SendTx(opt) => context.send_tx(&opt)?.try_into(),
        Balance(opt) => context.balance(&opt)?.try_into(),
        Rescan(opt) => Ok(context.rescan(&opt)?),
        ListCoins(opt) => context.list_coins(&opt)?.try_into(),
        Import(opt) => context.import(&opt),
    }
}
