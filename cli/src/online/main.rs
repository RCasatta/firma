use firma::bitcoin::Network;
use firma::bitcoincore_rpc::bitcoincore_rpc_json::bitcoin::blockdata::constants::genesis_block;
use firma::bitcoincore_rpc::RpcApi;
use firma::log::debug;
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
    Balance,

    /// View wallet coins
    ListCoins,
}

#[derive(StructOpt, Debug)]
pub struct ConnectOptions {
    #[structopt(flatten)]
    pub context: NewContext,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,
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

    if let Connect(ref opt) = &cmd.subcommand {
        let client = opt.daemon_opts.make_client(None)?;
        let genesis = client.get_block_hash(0)?;
        if genesis != genesis_block(opt.context.network).block_hash() {
            return Err(Error::IncompatibleNetworks);
        }
        let value = serde_json::to_value(&opt.daemon_opts)?;
        let vec = serde_json::to_vec_pretty(&opt.daemon_opts)?;
        let mut path = expand_tilde(&opt.context.firma_datadir)?;
        path.push(opt.context.network.to_string());
        std::fs::create_dir(&path)?;
        path.push("daemon_opts.json");
        debug!("writing daemon_opts.json in {:?}", path);
        std::fs::write(&path, vec)
            .map_err(|e| crate::Error::FileNotFoundOrCorrupt(path, e.to_string()))?;
        return Ok(value);
    }

    let daemon_opts = cmd.context.load_daemon_opts()?;

    let wallet = Wallet::new(
        daemon_opts.make_client(Some(cmd.context.wallet_name.to_string()))?,
        cmd.context.clone(),
    );

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
        CreateWallet(ref opt) => wallet.create(&opt, result.blocks)?.try_into(),
        GetAddress(ref opt) => wallet.get_address(opt)?.try_into(),
        CreateTx(ref opt) => wallet.create_tx(opt)?.try_into(),
        SendTx(ref opt) => wallet.send_tx(opt)?.try_into(),
        Balance => wallet.balance()?.try_into(),
        Rescan(ref opt) => Ok(wallet.rescan(opt)?),
        ListCoins => wallet.list_coins()?.try_into(),
        Connect(_) => unreachable!(),
    }
}
