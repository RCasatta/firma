use bitcoin::Network;
use firma::{init_logger, Result};
use log::debug;
use structopt::StructOpt;
use FirmaOfflineSubcommands::*;

mod dice;
mod print;
mod qr;
mod random;
mod restore;
mod sign;
mod derive_key;

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
    Dice(crate::dice::DiceOptions),

    /// Create a Master Private Key (xprv) with entropy from this machine RNG
    Random(crate::random::RandomOptions),

    /// Sign a PSBT with local Master Private Key (xprv)
    Sign(crate::sign::SignOptions),

    /// View a field in a json as qrcode shown in terminal
    Qr(crate::qr::QrOptions),

    /// Decode and print a PSBT
    Print(crate::print::PrintOptions),

    /// Restore a json key from xprv, hex seed or bech32 seed
    Restore(crate::restore::RestoreOptions),

    /// Hard derive a master key from a master^2 key
    DeriveKey(crate::derive_key::DeriveKeyOptions),
}

fn main() -> Result<()> {
    init_logger();
    let cmd = FirmaOfflineCommands::from_args();
    debug!("{:?}", cmd);

    let result = match cmd.subcommand {
        Dice(opt) => dice::roll(&cmd.firma_datadir, cmd.network, &opt),
        Sign(opt) => sign::start(&opt, cmd.network),
        Qr(opt) => qr::show(&opt),
        Random(opt) => random::start(&cmd.firma_datadir, cmd.network, &opt),
        Print(opt) => print::start(&opt, cmd.network),
        Restore(opt) => restore::start(&cmd.firma_datadir, cmd.network, &opt),
        DeriveKey(opt) => derive_key::start(&cmd.firma_datadir, cmd.network, &opt),
    };

    let value = match result {
        Ok(value) => value,
        Err(e) => e.to_json()?,
    };

    println!("{}", serde_json::to_string_pretty(&value)?);

    Ok(())
}
