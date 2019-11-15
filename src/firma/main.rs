use bitcoin::consensus::serialize;
use bitcoin::util::psbt::Map;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::Network;
use firma::PsbtJson;
use log::{debug, info};
use log::{Level, LevelFilter, Metadata, Record};
use sign::pretty_print;
use sign::psbt_from_base64;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

type PSBT = PartiallySignedTransaction;

pub mod sign;

static LOGGER: SimpleLogger = SimpleLogger;

/// Firma is a signer of Partially Signed Bitcoin Transaction (PSBT).
#[derive(StructOpt, Debug)]
#[structopt(name = "firma")]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Decode a PSBT and print informations
    #[structopt(short, long)]
    decode: bool,

    /// File containing the master key (xpriv...)
    #[structopt(short, long, parse(from_os_str))]
    key: Option<PathBuf>,

    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// derivations to consider if psbt doesn't contain HD paths
    #[structopt(short, long, default_value = "1000")]
    total_derivations: u32,

    /// PSBT json file
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let level = match opt.verbose {
        0 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
        .expect("cannot initialize logging");
    debug!("{:#?}", opt);
    let json = fs::read_to_string(&opt.file).unwrap();
    let mut json: PsbtJson = serde_json::from_str(&json).unwrap();
    debug!("{:#?}", json);

    let mut psbt = psbt_from_base64(&json.psbt)?;
    debug!("{:#?}", psbt);

    let initial_partial_sigs = get_partial_sigs(&psbt);

    if opt.decode {
        pretty_print(&psbt, opt.network)
    } else {
        if json.signed_psbt.is_some() || json.only_sigs.is_some() {
            info!("The json psbt already contain signed_psbt or only_sigs, exiting to avoid risk of overwriting data");
            return Ok(());
        }

        sign::start(&opt, &mut psbt, &mut json)?;

        let partial_sigs = get_partial_sigs(&psbt);

        if !partial_sigs.is_empty() {
            json.only_sigs = Some(base64::encode(&partial_sigs));
        }

        if initial_partial_sigs != partial_sigs {
            fs::write(&opt.file, serde_json::to_string_pretty(&json).unwrap())
                .unwrap_or_else(|_| panic!("Unable to write {:?}", &opt.file));
            info!("\nAdded signatures, wrote {:?}", &opt.file);
        } else {
            info!("\nNo signature added");
        }
    }

    Ok(())
}

fn get_partial_sigs(psbt: &PSBT) -> Vec<u8> {
    let mut only_partial_sigs = vec![];
    for input in psbt.inputs.iter() {
        for pair in input.get_pairs().unwrap().iter() {
            if pair.key.type_value == 2u8 {
                let vec = serialize(pair);
                debug!("partial sig pair {}", hex::encode(&vec));
                only_partial_sigs.extend(vec);
            }
        }
    }
    only_partial_sigs
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() <= Level::Warn {
                println!("{} - {}", record.level(), record.args());
            } else {
                println!("{}", record.args());
            }
        }
    }

    fn flush(&self) {}
}
