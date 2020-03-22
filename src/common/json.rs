use crate::DaemonOpts;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::{Address, OutPoint, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::WalletCreateFundedPsbtResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrivateMasterKey {
    pub xpub: ExtendedPubKey,
    pub xprv: ExtendedPrivKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launches: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyOutput {
    pub key: PrivateMasterKey,
    pub private_file: PathBuf,
    pub public_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicMasterKey {
    pub xpub: ExtendedPubKey,
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
    pub descriptor_main: String,
    pub descriptor_change: String,
    pub daemon_opts: DaemonOpts,
    pub stat: KindStat,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KindStat {
    pub kind: String,
    pub diffusion: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletIndexes {
    pub main: u32,
    pub change: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BalanceOutput {
    pub satoshi: u64,
    pub btc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetAddressOutput {
    pub address: Address,
    pub indexes: WalletIndexes,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SendTxOutput {
    pub hex: String,
    pub txid: Txid,
    pub broadcasted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateTxOutput {
    pub psbt_file: PathBuf,
    pub funded_psbt: WalletCreateFundedPsbtResult,
    pub address_reused: HashSet<Address>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateWalletOutput {
    pub wallet_file: PathBuf,
    pub wallet: WalletJson,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListCoinsOutput {
    pub coins: Vec<Coin>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Coin {
    pub outpoint: OutPoint,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorJson {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PsbtPrettyPrint {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub sizes: Vec<String>,
    pub fee: Fee,
    pub info: Vec<String>,
    pub psbt_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Fee {
    pub absolute: u64,
    pub rate: f64,
}
