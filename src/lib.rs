use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyJson {
    pub xpub: String,
    pub xpriv: String,
    pub launches: String,
    pub faces: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub psbt: String,
    pub fee: f64,
    pub changepos: u32,
    pub signed_psbt: Option<String>,
    pub only_sigs: Option<String>,
}
