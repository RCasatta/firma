use crate::dice::DiceOptions;
use crate::qr::QrOptions;
use crate::random::RandomOptions;
use crate::sign::SignOptions;
use bitcoin::Network;
use firma::init_logger;
use log::debug;
use std::error::Error;
use structopt::StructOpt;

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
    Dice(DiceOptions),
    Random(RandomOptions),
    Sign(SignOptions),
    Qr(QrOptions),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = FirmaOfflineCommands::from_args();

    init_logger(cmd.verbose);
    debug!("{:?}", cmd);

    match cmd.subcommand {
        FirmaOfflineSubcommands::Dice(opt) => dice::roll(&cmd.firma_datadir, cmd.network, &opt)?,
        FirmaOfflineSubcommands::Sign(opt) => sign::start(&opt)?,
        FirmaOfflineSubcommands::Qr(opt) => qr::show(&opt)?,
        FirmaOfflineSubcommands::Random(opt) => {
            random::start(&cmd.firma_datadir, cmd.network, &opt)?
        }
    }

    Ok(())
}
