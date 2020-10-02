use crate::ErrorJson;
use bitcoin::hashes::core::fmt::Formatter;
use serde_json::Value;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Generic(String),

    // Internal
    FileExist(PathBuf),
    DiceValueErr(u32, u32),
    WrongKeyFileName,
    MissingPrevoutTx,
    MismatchPrevoutHash,
    MissingDatadir,
    MissingNetwork,
    MissingDaemonOpts,
    MissingOutpoint,
    MissingTxout,
    MissingKey,
    MissingSighash,
    MissingWitnessUtxo,
    MissingAddress,
    MissingRescanUpTo,
    MissingHex,
    FileNotFoundOrCorrupt(PathBuf, String),
    MissingName,
    NeedAtLeastOne,
    CannotRetrieveHomeDir,
    AddressFromDescriptorFails,
    CaptureGroupNotFound(String),
    NonDefaultScript,
    ScriptEmpty,
    IncompatibleNetworks,
    Mnemonic(crate::common::mnemonic::Error),

    // Internal Qr
    QrAtLeast2Pieces,
    QrTotalMismatch(usize),
    QrMissingParts,
    QrParity,
    QrTooShort,
    QrStructuredWrongMode,
    QrStructuredWrongEnc,
    QrSeqGreaterThanTotal(u8, u8),
    QrLengthMismatch(usize, usize),
    QrUnsupportedVersion(i16),
    QrSplitMax16(usize),

    // External
    BitcoinRpc(bitcoincore_rpc::Error),
    BitcoinEncode(bitcoin::consensus::encode::Error),
    BitcoinKey(bitcoin::util::key::Error),
    BitcoinSecp256k1(bitcoin::secp256k1::Error),
    BitcoinPSBT(bitcoin::util::psbt::Error),
    BitcoinAddress(bitcoin::util::address::Error),
    BitcoinBech32(bitcoin::bech32::Error),
    BitcoinScriptError(bitcoin::blockdata::script::Error),
    Serde(serde_json::error::Error),
    IO(std::io::Error),
    Base58(bitcoin::util::base58::Error),
    Bip32(bitcoin::util::bip32::Error),
    Base64(base64::DecodeError),
    PathStrip(std::path::StripPrefixError),
    Qr(qrcode::types::QrError),
    Hex(hex::FromHexError),
    Env(std::env::VarError),
    Utf8(std::str::Utf8Error),
    Nul(std::ffi::NulError),
    Regex(regex::Error),
    ParseInt(std::num::ParseIntError),
    Miniscript(miniscript::Error),
    Bmp(bmp_monochrome::BmpError),
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err)
            }
        }
    };
}

impl_error!(bitcoincore_rpc::Error, BitcoinRpc);
impl_error!(bitcoin::util::base58::Error, Base58);
impl_error!(bitcoin::util::bip32::Error, Bip32);
impl_error!(bitcoin::consensus::encode::Error, BitcoinEncode);
impl_error!(bitcoin::util::key::Error, BitcoinKey);
impl_error!(bitcoin::secp256k1::Error, BitcoinSecp256k1);
impl_error!(bitcoin::util::psbt::Error, BitcoinPSBT);
impl_error!(bitcoin::util::address::Error, BitcoinAddress);
impl_error!(bitcoin::bech32::Error, BitcoinBech32);
impl_error!(bitcoin::blockdata::script::Error, BitcoinScriptError);
impl_error!(serde_json::error::Error, Serde);
impl_error!(std::io::Error, IO);
impl_error!(base64::DecodeError, Base64);
impl_error!(std::path::StripPrefixError, PathStrip);
impl_error!(qrcode::types::QrError, Qr);
impl_error!(hex::FromHexError, Hex);
impl_error!(std::env::VarError, Env);
impl_error!(std::str::Utf8Error, Utf8);
impl_error!(std::ffi::NulError, Nul);
impl_error!(regex::Error, Regex);
impl_error!(std::num::ParseIntError, ParseInt);
impl_error!(miniscript::Error, Miniscript);
impl_error!(crate::common::mnemonic::Error, Mnemonic);
impl_error!(bmp_monochrome::BmpError, Bmp);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Generic(e) => write!(f, "{}", e),

            Error::FileExist(s) => write!(f, "File {:?} already exist", s),
            Error::DiceValueErr(n, max) => write!(f, "Got {} but must be from 1 to {} included", n, max),
            Error::WrongKeyFileName=> write!(f, "Private file name MUST be PRIVATE.json"),
            Error::MissingPrevoutTx => write!(f, "Missing prevout tx"),
            Error::MismatchPrevoutHash => write!(f, "Prevout hash doesn't match previous tx"),
            Error::MissingDatadir => write!(f, "Missing datadir"),
            Error::MissingNetwork => write!(f, "Missing network"),
            Error::MissingDaemonOpts => write!(f, "Missing daemon options (url and cookie file)"),
            Error::FileNotFoundOrCorrupt(p, e) => write!(f, "{:?} file not found or corrupted: {}", p, e),
            Error::MissingName => write!(f, "Missing name"),
            Error::NeedAtLeastOne => write!(f, "Need at least one"),
            Error::CannotRetrieveHomeDir => write!(f, "Cannot retrieve home dir"),
            Error::AddressFromDescriptorFails => write!(f, "can't create address from descriptor"),
            Error::CaptureGroupNotFound(s) => write!(f, "Capture group of {} not found", s),
            Error::NonDefaultScript => write!(f, "Non default script"),
            Error::ScriptEmpty => write!(f, "Script empty"),
            Error::MissingOutpoint => write!(f, "Missing outpoint"),
            Error::MissingTxout => write!(f, "Missing Txout"),
            Error::MissingKey => write!(f, "Missing Key"),
            Error::MissingSighash => write!(f, "Missing Sighash"),
            Error::MissingWitnessUtxo => write!(f, "Missing Witness UTXO"),
            Error::MissingAddress => write!(f, "Missing Address"),
            Error::MissingRescanUpTo => write!(f, "Missing RescanUpTo"),
            Error::MissingHex => write!(f, "Missing hex"),
            Error::IncompatibleNetworks => write!(f, "Incompatible networks"),

            Error::QrAtLeast2Pieces => write!(f, "Need at least 2 different pieces to merge structured QR"),
            Error::QrTotalMismatch(i) => write!(f, "Total pieces in input {} does not match the encoded total, or different encoded totals", i ),
            Error::QrMissingParts => write!(f, "Not all the part are present"),
            Error::QrParity => write!(f, "Invalid parities while merging"),
            Error::QrTooShort => write!(f, "QR data shorter than 5 bytes"),
            Error::QrStructuredWrongMode => write!(f, "Structured append QR must have mode 3"),
            Error::QrStructuredWrongEnc => write!(f, "Structured append QR must have encoding 4"),
            Error::QrSeqGreaterThanTotal(s, t) => write!(f,  "QR sequence {} greater than total {}",s, t ),
            Error::QrLengthMismatch(calc, exp) => write!(f,  "calculated end {} greater than effective length {}", calc, exp ),
            Error::QrUnsupportedVersion(ver) => write!(f,  "Unsupported version {}", ver),
            Error::QrSplitMax16(req) => write!(f,  "Could split into max 16 qr, requested {}", req),

            Error::BitcoinRpc(e) => write!(f, "{:?}", e),
            Error::Serde(e) => write!(f, "{:?}", e),
            Error::IO(e) => write!(f, "{:?}", e),
            Error::Base58(e) => write!(f, "{:?}", e),
            Error::Bip32(e) => write!(f, "{:?}", e),
            Error::Base64(e) => write!(f, "{:?}", e),
            Error::BitcoinEncode(e) => write!(f, "{:?}", e),
            Error::BitcoinKey(e) => write!(f, "{:?}", e),
            Error::BitcoinSecp256k1(e) => write!(f, "{:?}", e),
            Error::BitcoinPSBT(e) => write!(f, "{:?}", e),
            Error::BitcoinAddress(e) => write!(f, "{:?}", e),
            Error::BitcoinBech32(e) => write!(f, "{:?}", e),
            Error::BitcoinScriptError(e) => write!(f, "{:?}", e),
            Error::PathStrip(e) => write!(f, "{:?}", e),
            Error::Qr(e) => write!(f, "{:?}", e),
            Error::Hex(e) => write!(f, "{:?}", e),
            Error::Env(e) => write!(f, "{:?}", e),
            Error::Utf8(e) => write!(f, "{:?}", e),
            Error::Nul(e) => write!(f, "{:?}", e),
            Error::Regex(e) => write!(f, "{:?}", e),
            Error::ParseInt(e) => write!(f, "{:?}", e),
            Error::Miniscript(e) => write!(f, "{:?}", e),
            Error::Mnemonic(e) => write!(f, "{:?}", e),
            Error::Bmp(e) => write!(f, "{:?}", e),
        }
    }
}

pub fn io_err(str: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, str.to_string())
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Generic(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Error {
        Error::Generic(e.to_string())
    }
}

impl Error {
    pub fn to_json(&self) -> Value {
        let value = ErrorJson {
            error: self.to_string(),
        };
        serde_json::to_value(&value).unwrap() // safe to unwrap, ErrorJson does not contain map with non string keys
    }
}
