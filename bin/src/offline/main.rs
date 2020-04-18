use bitcoin::Network;
use firma::{common, init_logger, offline, Result};
use serde_json::Value;
use std::convert::TryInto;
use structopt::StructOpt;
use FirmaOfflineSubcommands::*;

/// firma-offline is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-offline")]
struct FirmaOfflineCommands {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    firma_datadir: String,

    //TODO ContextOffline with network, json, firma_datadir
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    subcommand: FirmaOfflineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOfflineSubcommands {
    /// Create a Master Private Key (xprv) with entropy from dice launches
    Dice(offline::dice::DiceOptions),

    /// Create a Master Private Key (xprv) with entropy from this machine RNG
    Random(offline::random::RandomOptions),

    /// Sign a PSBT with local Master Private Key (xprv)
    Sign(offline::sign::SignOptions),

    /// Decode and print a PSBT
    Print(offline::print::PrintOptions),

    /// Restore a json key from xprv, hex seed or bech32 seed
    Restore(offline::restore::RestoreOptions),

    /// List wallets and keys
    List(common::list::ListOptions),

    /// Hard derive a master key from a master^2 key
    DeriveKey(offline::derive_key::DeriveKeyOptions),
}

fn main() -> Result<()> {
    init_logger();
    let cmd = FirmaOfflineCommands::from_args();

    let value = match launch_subcommand(&cmd) {
        Ok(value) => value,
        Err(e) => e.to_json(),
    };

    println!("{}", serde_json::to_string_pretty(&value)?);

    Ok(())
}

fn launch_subcommand(cmd: &FirmaOfflineCommands) -> Result<Value> {
    let net = cmd.network;
    let datadir = &cmd.firma_datadir;
    match &cmd.subcommand {
        Dice(opt) => offline::dice::roll(datadir, net, &opt)?.try_into(),
        Sign(opt) => offline::sign::start(&opt, net)?.try_into(),
        Random(opt) => offline::random::create_key(datadir, net, &opt)?.try_into(),
        Print(opt) => offline::print::start(datadir, net, &opt)?.try_into(),
        Restore(opt) => offline::restore::start(datadir, net, &opt)?.try_into(),
        DeriveKey(opt) => offline::derive_key::start(datadir, net, &opt)?.try_into(),
        List(opt) => common::list::list(datadir, net, &opt)?.try_into(),
    }
}
