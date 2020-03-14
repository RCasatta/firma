use crate::DaemonOpts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrivateMasterKeyJson {
    pub xpub: String,
    pub xpriv: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launches: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicMasterKeyJson {
    pub xpub: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub psbt: String,
    pub fee: f64,
    pub changepos: i32,
    pub signed_psbt: Option<String>,
    pub only_sigs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletJson {
    pub name: String,
    pub main_descriptor: String,
    pub change_descriptor: String,
    pub daemon_opts: DaemonOpts,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletIndexesJson {
    pub main: u32,
    pub change: u32,
}
