use firma::online::PathOptions;
use firma::serde_json::{self, Value};
use firma::{common, init_logger, offline, Context, Error, Result, StringEncoding, ToJson};
use std::convert::TryInto;
use std::io;
use std::io::Read;
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

    /// Flag to indicate that input is expected in standard input.
    /// Since reading stdin is locking, we need this flag to have it optionally
    #[structopt(long)]
    pub read_stdin: bool,
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
    Decrypt(offline::decrypt::DecryptOptions),

    /// Sign a wallet json containing the descriptor to avoid tampering
    SignWallet(offline::sign_wallet::SignWalletOptions),

    /// Verify a wallet json containing the descriptor to avoid tampering
    VerifyWallet(offline::sign_wallet::VerifyWalletOptions),

    /// Import the file containing a firma json object
    Import(PathOptions),
}

fn main() -> Result<()> {
    init_logger();
    let mut cmd = FirmaOfflineCommands::from_args();

    if cmd.read_stdin {
        // read encryption key from stdin and initialize encryption_key field
        let mut buffer = vec![];
        io::stdin().read_to_end(&mut buffer)?;
        let encoded = StringEncoding::new_base64(&buffer);
        match &mut cmd.subcommand {
            Random(opt) => opt.encryption_key = Some(encoded),
            Sign(opt) => opt.encryption_key = Some(encoded),
            Decrypt(opt) => opt.encryption_key = Some(encoded),
            Dice(opt) => opt.encryption_key = Some(encoded),
            Restore(opt) => opt.encryption_key = Some(encoded),
            SignWallet(opt) => opt.encryption_key = Some(encoded),
            List(opt) => opt.encryption_keys = vec![encoded],
            _ => {
                let err = Error::Generic("Subcommand doesn't need encryption key".to_string());
                println!("{}", serde_json::to_string_pretty(&err.to_json())?);
                return Ok(());
            }
        }
    }

    let value = match launch_subcommand(&cmd) {
        Ok(value) => value,
        Err(e) => e.to_json(),
    };

    println!("{}", serde_json::to_string_pretty(&value)?);

    Ok(())
}

fn launch_subcommand(cmd: &FirmaOfflineCommands) -> Result<Value> {
    let FirmaOfflineCommands {
        context,
        subcommand,
        read_stdin: _,
    } = cmd;
    match &subcommand {
        Dice(opt) => context.roll(opt)?.try_into(),
        Sign(opt) => context.sign(opt)?.try_into(),
        Random(opt) => context.create_key(opt)?.try_into(),
        Print(opt) => context.print(opt)?.try_into(),
        Restore(opt) => offline::restore::start(context, opt)?.try_into(),
        List(opt) => context.list(opt)?.try_into(),
        SignWallet(opt) => context.sign_wallet(opt)?.try_into(),
        VerifyWallet(opt) => context.verify_wallet(opt)?.try_into(),
        Import(opt) => context.import(opt),
        Decrypt(opt) => offline::decrypt::decrypt::<Value>(opt),
    }
}
