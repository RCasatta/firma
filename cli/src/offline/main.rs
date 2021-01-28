use firma::log::debug;
use firma::online::PathOptions;
use firma::serde_json::{self, Value};
use firma::{common, init_logger, offline, Context, Result, ToJson};
use std::convert::TryInto;
use structopt::StructOpt;
use FirmaOfflineSubcommands::*;

/// firma-offline is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma-offline")]
struct FirmaOfflineCommands {
    #[structopt(flatten)]
    context: Context,

    #[structopt(subcommand)]
    subcommand: FirmaOfflineSubcommands,

    /// Flag to indicate usage of encryption/decryption when using CLI
    /// when true, reading from stdin is expected and blocking
    #[structopt(short, long)]
    encrypt: bool,
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

    /// Restore a json key from xprv or mnemonic
    Restore(offline::restore::RestoreOptions),

    /// List wallets and keys
    List(common::list::ListOptions),

    /// Decrypt an encrypted file
    Decrypt(PathOptions),

    /// Sign a wallet json containing the descriptor to avoid tampering
    SignWallet(offline::sign_wallet::SignWalletOptions),

    /// Verify a wallet json containing the descriptor to avoid tampering
    VerifyWallet(offline::sign_wallet::VerifyWalletOptions),

    /// Import the file containing a firma json object
    Import(PathOptions),
}

fn main() -> Result<()> {
    init_logger();
    debug!("firma-offline start");
    let cmd = FirmaOfflineCommands::from_args();
    let FirmaOfflineCommands {
        mut context,
        subcommand,
        encrypt,
    } = cmd;

    if encrypt {
        context.read_encryption_key()?;
    }

    debug!(
        "firma-offline context:{:?} encrypt:{} subcommand:{:?}",
        context, encrypt, subcommand
    );

    let value = match launch_subcommand(&context, subcommand) {
        Ok(value) => value,
        Err(e) => e.to_json(),
    };

    println!("{}", serde_json::to_string_pretty(&value)?);

    Ok(())
}

fn launch_subcommand(context: &Context, subcommand: FirmaOfflineSubcommands) -> Result<Value> {
    match &subcommand {
        Dice(opt) => context.roll(opt)?.try_into(),
        Sign(opt) => context.sign(opt)?.try_into(),
        Random(opt) => context.create_key(opt)?.try_into(),
        Print(opt) => context.print(opt)?.try_into(),
        Restore(opt) => context.restore(opt)?.try_into(),
        List(opt) => context.list(opt)?.try_into(),
        SignWallet(opt) => context.sign_wallet(opt)?.try_into(),
        VerifyWallet(opt) => context.verify_wallet(opt)?.try_into(),
        Import(opt) => context.import(opt),
        Decrypt(opt) => offline::decrypt::decrypt::<Value>(opt, &context.encryption_key),
    }
}
