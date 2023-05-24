use crate::ErrorJson;
use bitcoin::BlockHash;
use core::fmt::Formatter;
use miniscript::descriptor;
use qr_code::types::QrError;
use serde_json::Value;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Generic(String),

    // Internal
    FileExist(PathBuf),
    DiceValueErr(u32, u32),
    MissingPrevoutTx,
    MismatchPrevoutHash,
    MissingContext,
    MissingMethod,
    MethodNotExist(String),
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
    CannotOverwrite(PathBuf),
    MissingName,
    NeedAtLeastOne,
    CannotRetrieveHomeDir,
    AddressFromDescriptorFails,
    CaptureGroupNotFound(String),
    NonDefaultScript,
    ScriptEmpty,
    IncompatibleNetworks,
    IncompatibleGenesis { node: BlockHash, firma: BlockHash },
    Mnemonic(crate::common::mnemonic::Error),
    PsbtNotChangedAfterMerge,
    PsbtBadStringEncoding(String),
    MaybeEncryptedWrongState,
    EncryptionKeyNot32Bytes(usize),
    MissingEncryptionKey,
    InvalidMessageSignature,
    MissingIdentifier,
    WalletNotExistsInNode(String),
    WalletAlreadyExistsInNode(String),
    WalletSignatureNotVerified,
    WrongKeyType,
    MissingUtxoAndNotFinalized,

    // External
    BitcoinRpc(bitcoincore_rpc::Error),
    BitcoinEncode(bitcoin::consensus::encode::Error),
    BitcoinKey(bitcoin::util::key::Error),
    BitcoinSecp256k1(bitcoin::secp256k1::Error),
    BitcoinPsbt(bitcoin::util::psbt::Error),
    BitcoinAddress(bitcoin::util::address::Error),
    BitcoinBech32(bitcoin::bech32::Error),
    BitcoinScriptError(bitcoin::blockdata::script::Error),
    Serde(serde_json::error::Error),
    Io(std::io::Error),
    Base58(bitcoin::util::base58::Error),
    Bip32(bitcoin::util::bip32::Error),
    Base64(base64::DecodeError),
    PathStrip(std::path::StripPrefixError),
    Qr(qr_code::types::QrError),
    Hex(hex::FromHexError),
    Env(std::env::VarError),
    Utf8(std::str::Utf8Error),
    Nul(std::ffi::NulError),
    ParseInt(std::num::ParseIntError),
    Miniscript(miniscript::Error),
    MiniscriptDescriptor(descriptor::DescriptorKeyParseError),
    MiniscriptConversion(descriptor::ConversionError),
    Bmp(qr_code::bmp_monochrome::BmpError),
    Encryption(aes_gcm_siv::aead::Error),
    PsbtCannotDeserialize(bitcoin::consensus::encode::Error),
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
impl_error!(bitcoin::util::psbt::Error, BitcoinPsbt);
impl_error!(bitcoin::util::address::Error, BitcoinAddress);
impl_error!(bitcoin::bech32::Error, BitcoinBech32);
impl_error!(bitcoin::blockdata::script::Error, BitcoinScriptError);
impl_error!(serde_json::error::Error, Serde);
impl_error!(std::io::Error, Io);
impl_error!(base64::DecodeError, Base64);
impl_error!(std::path::StripPrefixError, PathStrip);
impl_error!(qr_code::types::QrError, Qr);
impl_error!(hex::FromHexError, Hex);
impl_error!(std::env::VarError, Env);
impl_error!(std::str::Utf8Error, Utf8);
impl_error!(std::ffi::NulError, Nul);
impl_error!(std::num::ParseIntError, ParseInt);
impl_error!(miniscript::Error, Miniscript);
impl_error!(crate::common::mnemonic::Error, Mnemonic);
impl_error!(qr_code::bmp_monochrome::BmpError, Bmp);
impl_error!(aes_gcm_siv::aead::Error, Encryption);
impl_error!(descriptor::DescriptorKeyParseError, MiniscriptDescriptor);
impl_error!(descriptor::ConversionError, MiniscriptConversion);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Generic(e) => write!(f, "{}", e),

            Error::FileExist(s) => write!(f, "File {:?} already exist", s),
            Error::DiceValueErr(n, max) => {
                write!(f, "Got {} but must be from 1 to {} included", n, max)
            }
            Error::MissingPrevoutTx => write!(f, "Missing prevout tx"),
            Error::MismatchPrevoutHash => write!(f, "Prevout hash doesn't match previous tx"),
            Error::MissingContext => write!(f, "Missing context"),
            Error::MissingMethod => write!(f, "Missing method"),
            Error::MethodNotExist(m) => write!(f, "Method {} not exist", m),
            Error::MissingDaemonOpts => write!(f, "Missing daemon options (url and cookie file)"),
            Error::FileNotFoundOrCorrupt(p, e) => {
                write!(f, "{:?} file not found or corrupted: {}", p, e)
            }
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
            Error::IncompatibleGenesis { node, firma } => write!(
                f,
                "Incompatible genesis block node:{} firma:{}",
                node, firma
            ),
            Error::PsbtNotChangedAfterMerge => write!(f, "PSBT did not change after merge"),
            Error::PsbtBadStringEncoding(kind) => {
                write!(f, "PSBT has bad {} string encoding", kind)
            }
            Error::PsbtCannotDeserialize(e) => write!(f, "Cannot deserialize PSBT ({})", e),
            Error::MaybeEncryptedWrongState => write!(f, "Wrong State"),
            Error::Encryption(e) => write!(f, "Encryption ({})", e),
            Error::EncryptionKeyNot32Bytes(s) => {
                write!(f, "Encryption key must be 32 bytes but it's {} bytes", s)
            }
            Error::MissingEncryptionKey => write!(f, "MissingEncryptionKey"),
            Error::InvalidMessageSignature => write!(f, "Invalid message signature"),
            Error::CannotOverwrite(p) => write!(f, "Cannot overwrite {:?}", p),
            Error::MissingIdentifier => write!(f, "Missing identifier"),
            Error::WalletNotExistsInNode(s) => {
                write!(f, "Wallet {} does not exist in the bitcoin node", s)
            }
            Error::WalletAlreadyExistsInNode(s) => {
                write!(f, "Wallet {} already exists in the bitcoin node", s)
            }
            Error::WalletSignatureNotVerified => write!(
                f,
                "The wallet signature did not verify with any of the key of the wallet"
            ),
            Error::WrongKeyType => write!(f, "Expected another key type"),
            Error::MissingUtxoAndNotFinalized => {
                write!(f, "witness_utxo and non_witness_utxo are both None")
            }

            Error::BitcoinRpc(e) => write!(f, "{:?}", e),
            Error::Serde(e) => write!(f, "{:?}", e),
            Error::Io(e) => write!(f, "{:?}", e),
            Error::Base58(e) => write!(f, "{:?}", e),
            Error::Bip32(e) => write!(f, "{:?}", e),
            Error::Base64(e) => write!(f, "{:?}", e),
            Error::BitcoinEncode(e) => write!(f, "{:?}", e),
            Error::BitcoinKey(e) => write!(f, "{:?}", e),
            Error::BitcoinSecp256k1(e) => write!(f, "{:?}", e),
            Error::BitcoinPsbt(e) => write!(f, "{:?}", e),
            Error::BitcoinAddress(e) => write!(f, "{:?}", e),
            Error::BitcoinBech32(e) => write!(f, "{:?}", e),
            Error::BitcoinScriptError(e) => write!(f, "{:?}", e),
            Error::PathStrip(e) => write!(f, "{:?}", e),
            Error::Qr(e) => write!(f, "{:?}", e),
            Error::Hex(e) => write!(f, "{:?}", e),
            Error::Env(e) => write!(f, "{:?}", e),
            Error::Utf8(e) => write!(f, "{:?}", e),
            Error::Nul(e) => write!(f, "{:?}", e),
            Error::ParseInt(e) => write!(f, "{:?}", e),
            Error::Miniscript(e) => write!(f, "{:?}", e),
            Error::MiniscriptDescriptor(e) => write!(f, "{:?}", e),
            Error::MiniscriptConversion(e) => write!(f, "{:?}", e),
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

pub trait ToJson {
    fn to_json(&self) -> Value;
}

impl ToJson for Error {
    fn to_json(&self) -> Value {
        let value = ErrorJson {
            error: self.to_string(),
        };
        serde_json::to_value(value).unwrap() // safe to unwrap, ErrorJson does not contain map with non string keys
    }
}

impl ToJson for QrError {
    fn to_json(&self) -> Value {
        let value = ErrorJson {
            error: self.to_string(),
        };
        serde_json::to_value(value).unwrap() // safe to unwrap, ErrorJson does not contain map with non string keys
    }
}
