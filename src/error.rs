
#[derive(Debug)]
pub struct Error(pub String);

impl From<bitcoincore_rpc::Error> for Error {
    fn from(e: bitcoincore_rpc::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Error {
        Error(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<bitcoin::util::base58::Error> for Error {
    fn from(e: bitcoin::util::base58::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<bitcoin::util::bip32::Error> for Error {
    fn from(e: bitcoin::util::bip32::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Error {
        Error(e.to_string())
    }
}

impl From<bitcoin::consensus::encode::Error> for Error {
    fn from(e: bitcoin::consensus::encode::Error) -> Error {
        Error(e.to_string())
    }
}
