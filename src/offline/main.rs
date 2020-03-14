use crate::dice::DiceOptions;
use crate::qr::QrOptions;
use crate::random::RandomOptions;
use crate::sign::SignOptions;
use bitcoin::Network;
use firma::{init_logger, Result};
use log::{debug, error};
use structopt::StructOpt;
use FirmaOfflineSubcommands::*;

mod dice;
mod qr;
mod random;
mod sign;

/// firma-offline is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-offline")]
struct FirmaOfflineCommands {
    /// Verbose mode (-v)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    firma_datadir: String,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    subcommand: FirmaOfflineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOfflineSubcommands {
    /// Create a Master Private Key (xprv) with entropy from dice launches
    Dice(DiceOptions),

    /// Create a Master Private Key (xprv) with entropy from this machine RNG
    Random(RandomOptions),

    /// Sign a PSBT with local Master Private Key (xprv)
    Sign(SignOptions),

    /// View a field in a json as qrcode shown in terminal
    Qr(QrOptions),
}

fn main() -> Result<()> {
    let cmd = FirmaOfflineCommands::from_args();

    init_logger(cmd.verbose);
    debug!("{:?}", cmd);

    let result = match cmd.subcommand {
        Dice(opt) => dice::roll(&cmd.firma_datadir, cmd.network, &opt),
        Sign(opt) => sign::start(&opt),
        Qr(opt) => qr::show(&opt),
        Random(opt) => random::start(&cmd.firma_datadir, cmd.network, &opt),
    };

    if let Err(error) = result {
        error!("{}", error.0);
    }

    Ok(())
}
