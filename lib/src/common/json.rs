use crate::DaemonOpts;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::{bech32, Address, Network, OutPoint, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::WalletCreateFundedPsbtResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::convert::TryInto;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrivateMasterKey {
    pub xpub: ExtendedPubKey,
    pub xprv: ExtendedPrivKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<Seed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
    pub name: String,
    pub fingerprint: Fingerprint,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dice {
    pub launches: String,
    pub faces: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Seed {
    pub hex: String,
    pub bech32: String,
    pub network: Network,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyOutput {
    pub key: PrivateMasterKey,
    pub private_file: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_file: Option<PathBuf>,
    pub public_qr_files: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicMasterKey {
    pub xpub: ExtendedPubKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub name: String,
    pub psbt: String,
    pub fee: f64,
    pub changepos: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJsonOutput {
    pub psbt: PsbtJson,
    pub file: PathBuf,
    pub qr_files: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletJson {
    pub name: String,
    pub descriptor_main: String,
    pub descriptor_change: String,
    pub fingerprints: HashSet<Fingerprint>,
    pub required_sig: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_opts: Option<DaemonOpts>,
    pub created_at_height: u64,
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
    pub funded_psbt: PsbtJson,
    pub address_reused: HashSet<Address>,
    pub qr_files: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateWalletOutput {
    pub qr_files: Vec<PathBuf>,
    pub wallet_file: PathBuf,
    pub wallet: WalletJson,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ListOutput {
    pub keys: Vec<MasterKeyOutput>,
    pub wallets: Vec<CreateWalletOutput>,
    pub psbts: Vec<PsbtJsonOutput>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TxInOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    pub value: String,
    pub path: String,
    pub wallet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PsbtPrettyPrint {
    pub inputs: Vec<TxInOut>,
    pub outputs: Vec<TxInOut>,
    pub size: Size,
    pub fee: Fee,
    pub info: Vec<String>,
    pub psbt_file: PathBuf,
    pub balances: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Fee {
    pub absolute: u64,
    pub absolute_fmt: String,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Size {
    pub unsigned: usize,
    pub estimated: usize,
    pub psbt: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateQrOptions {
    pub path: PathBuf,
    pub qr_version: i16,
}

impl PsbtJson {
    pub fn from_rpc(psbt: WalletCreateFundedPsbtResult, name: &str) -> Self {
        PsbtJson {
            psbt: psbt.psbt,
            fee: psbt.fee.as_btc(),
            changepos: psbt.change_position,
            name: name.to_string(),
        }
    }
}

impl Seed {
    pub fn new(sec: &[u8], network: Network) -> crate::Result<Seed> {
        let hex = hex::encode(&sec);
        let bech32 = bech32::encode(&network.to_hrp(), bech32::ToBase32::to_base32(&sec))?;
        Ok(Seed {
            hex,
            bech32,
            network,
        })
    }
}

impl MasterKeyOutput {
    pub fn public_file_str(&self) -> Option<String> {
        self.public_file
            .as_ref()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    }

    pub fn private_file_str(&self) -> Option<String> {
        self.private_file.to_str().map(|s| s.to_string())
    }
}

impl PrivateMasterKey {
    pub fn new(network: Network, sec: &[u8], name: &str) -> crate::Result<PrivateMasterKey> {
        let secp = bitcoin::secp256k1::Secp256k1::signing_only();

        let xprv = ExtendedPrivKey::new_master(network, &sec)?;
        let xpub = ExtendedPubKey::from_private(&secp, &xprv);
        let seed = Some(Seed::new(&sec, network)?);

        Ok(PrivateMasterKey {
            xprv,
            xpub,
            seed,
            dice: None,
            name: name.to_string(),
            fingerprint: xpub.fingerprint(),
        })
    }

    pub fn from_xprv(xprv: ExtendedPrivKey, name: &str) -> Self {
        let secp = bitcoin::secp256k1::Secp256k1::signing_only();
        let xpub = ExtendedPubKey::from_private(&secp, &xprv);
        PrivateMasterKey {
            xprv,
            xpub,
            seed: None,
            dice: None,
            name: name.to_string(),
            fingerprint: xpub.fingerprint(),
        }
    }
}

pub trait ToHrp {
    fn to_hrp(&self) -> String;
}

impl ToHrp for Network {
    fn to_hrp(&self) -> String {
        match self {
            Network::Bitcoin => "bs",
            Network::Testnet => "ts",
            Network::Regtest => "rs",
        }
        .to_string()
    }
}

macro_rules! impl_try_into {
    ( $for:ty ) => {
        impl TryInto<Value> for $for {
            type Error = crate::Error;

            fn try_into(self) -> Result<Value, Self::Error> {
                Ok(serde_json::to_value(self)?)
            }
        }
    };
}
impl_try_into!(MasterKeyOutput);
impl_try_into!(PsbtPrettyPrint);
impl_try_into!(CreateWalletOutput);
impl_try_into!(CreateTxOutput);
impl_try_into!(SendTxOutput);
impl_try_into!(BalanceOutput);
impl_try_into!(ListCoinsOutput);
impl_try_into!(GetAddressOutput);
impl_try_into!(ListOutput);
