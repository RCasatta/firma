#[derive(Debug)]
pub struct Error(pub String);

pub fn err(str: &str) -> impl Fn() -> Error + '_ {
    move || Error(str.into())
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error(e)
    }
}

macro_rules! impl_error {
    ( $from:ty ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error(err.to_string())
            }
        }
    };
}

impl_error!(bitcoincore_rpc::Error);
impl_error!(&str);
impl_error!(serde_json::error::Error);
impl_error!(std::io::Error);
impl_error!(bitcoin::util::base58::Error);
impl_error!(bitcoin::util::bip32::Error);
impl_error!(base64::DecodeError);
impl_error!(bitcoin::consensus::encode::Error);
impl_error!(std::path::StripPrefixError);
impl_error!(qrcode::types::QrError);
impl_error!(bitcoin::util::key::Error);
impl_error!(bitcoin::secp256k1::Error);
impl_error!(bitcoin::util::psbt::Error);
