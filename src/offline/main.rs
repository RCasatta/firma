use crate::dice::DiceOptions;
use crate::qr::QrOptions;
use crate::sign::SignOptions;
use bitcoin::Network;
use firma::init_logger;
use std::error::Error;
use structopt::StructOpt;

mod dice;
mod qr;
mod sign;

/// firma-offline is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-offline")]
struct FirmaOfflineCommands {
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

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    subcommand: FirmaOfflineSubcommands,
}

#[derive(StructOpt, Debug)]
enum FirmaOfflineSubcommands {
    Dice(DiceOptions),
    Sign(SignOptions),
    Qr(QrOptions),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = FirmaOfflineCommands::from_args();
    println!("{:?}", cmd);

    init_logger(cmd.verbose);

    match cmd.subcommand {
        FirmaOfflineSubcommands::Dice(opt) => {
            dice::roll(&cmd.firma_datadir, &cmd.wallet_name, &cmd.network, &opt)?
        }
        FirmaOfflineSubcommands::Sign(opt) => sign::start(&opt)?,
        FirmaOfflineSubcommands::Qr(opt) => qr::show(&opt)?,
    }

    Ok(())
}
