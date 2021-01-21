pub mod identifier;

use crate::common::json::identifier::{IdKind, Identifier};
use crate::common::mnemonic::Mnemonic;
use crate::offline::sign::get_psbt_name;
use crate::{psbt_from_base64, psbt_to_base64, PSBT};
use bitcoin::bech32::FromBase32;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::util::psbt::{raw, Map};
use bitcoin::{bech32, Address, Amount, Network, OutPoint, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::WalletCreateFundedPsbtResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::convert::TryInto;
use std::path::PathBuf;

//TODO remove json suffix, use it with json namespace

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletJson {
    pub id: Identifier,
    pub descriptor: String,
    pub fingerprints: BTreeSet<Fingerprint>, // TODO derive from descriptor, or include in descriptor?
    pub required_sig: usize,                 // TODO derive from descriptor?
    pub created_at_height: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IndexesJson {
    pub id: Identifier,
    pub main: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletSignatureJson {
    pub id: Identifier,
    pub xpub: ExtendedPubKey,
    pub address: Address,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterSecretJson {
    pub id: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<Mnemonic>,
    pub xpub: ExtendedPubKey,
    pub xprv: ExtendedPrivKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
    pub fingerprint: Fingerprint,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicMasterKey {
    //TODO make it DescriptorPublicKeyJson
    pub id: Identifier,
    pub xpub: ExtendedPubKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub id: Identifier,
    pub psbt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dice {
    pub launches: String,
    pub faces: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyOutput {
    pub key: MasterSecretJson,
    pub private_file: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_file: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJsonOutput {
    pub psbt: PsbtJson,
    pub file: PathBuf,
    pub signatures: String,
    pub qr_files: Vec<PathBuf>,
    pub unsigned_txid: Txid,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BalanceSatBtc {
    pub satoshi: u64,
    pub btc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BalanceOutput {
    pub confirmed: BalanceSatBtc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<BalanceSatBtc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetAddressOutput {
    pub address: Address,
    pub path: DerivationPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SendTxOutput {
    pub hex: String,
    pub txid: Txid,
    pub broadcasted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateTxOutput {
    pub psbt_name: String,
    pub funded_psbt: PsbtJson,
    pub address_reused: HashSet<Address>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateWalletOutput {
    pub wallet_file: PathBuf,
    pub wallet: WalletJson,
    pub signature: Option<WalletSignatureJson>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unconfirmed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorJson {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TxIn {
    pub outpoint: String,
    pub signatures: HashSet<Fingerprint>,
    #[serde(flatten)]
    pub common: TxCommonInOut,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TxOut {
    pub address: String,
    #[serde(flatten)]
    pub common: TxCommonInOut,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TxCommonInOut {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_with_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PsbtPrettyPrint {
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
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
    pub rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Size {
    pub unsigned: usize,
    pub estimated: Option<usize>,
    pub psbt: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SavePSBTOptions {
    pub psbt: StringEncoding,
    pub qr_version: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct VerifyWalletResult {
    pub descriptor: String,
    pub signature: WalletSignatureJson,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "t", content = "c", rename_all = "lowercase")]
pub enum StringEncoding {
    Base64(String),
    Hex(String),
    Bech32(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Balance {
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub trusted: Amount,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub untrusted_pending: Amount,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub immature: Amount,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Balances {
    pub mine: Balance,
    pub watchonly: Balance,
}

impl StringEncoding {
    pub fn new_base64(content: &[u8]) -> Self {
        StringEncoding::Base64(base64::encode(content))
    }

    pub fn as_bytes(&self) -> crate::Result<Vec<u8>> {
        Ok(match self {
            StringEncoding::Base64(s) => base64::decode(s)?,
            StringEncoding::Hex(s) => hex::decode(s)?,
            StringEncoding::Bech32(s) => {
                let (_, vec_u5) = bech32::decode(s)?;
                Vec::<u8>::from_base32(&vec_u5)?
            }
        })
    }

    pub fn get_exactly_32(&self) -> crate::Result<[u8; 32]> {
        let bytes = self.as_bytes()?;
        if bytes.len() != 32 {
            return Err(crate::Error::EncryptionKeyNot32Bytes(bytes.len()));
        }
        let mut result = [0u8; 32];
        result.copy_from_slice(&bytes[..]);
        Ok(result)
    }

    pub fn kind(&self) -> String {
        match self {
            StringEncoding::Base64(_) => "base64",
            StringEncoding::Hex(_) => "hex",
            StringEncoding::Bech32(_) => "bech32",
        }
        .to_string()
    }
}

pub fn get_name_key() -> raw::Key {
    raw::Key {
        type_value: 0xFC,
        key: b"name".to_vec(),
    }
}

pub fn psbt_from_rpc(psbt: &WalletCreateFundedPsbtResult, name: &str) -> crate::Result<PSBT> {
    let (_, mut psbt_with_name) = psbt_from_base64(&psbt.psbt)?;

    let pair = raw::Pair {
        key: get_name_key(),
        value: name.as_bytes().to_vec(),
    };
    psbt_with_name.global.insert_pair(pair)?;
    Ok(psbt_with_name)
}

impl From<(&PSBT, Network)> for PsbtJson {
    fn from(psbt_and_network: (&PSBT, Network)) -> Self {
        let (psbt, network) = psbt_and_network;
        let (_, base64) = psbt_to_base64(psbt);
        let name = get_psbt_name(psbt).expect("PSBT without name"); //TODO
        PsbtJson {
            psbt: base64,
            id: Identifier::new(network, IdKind::PSBT, &name),
        }
    }
}

impl PsbtJson {
    pub fn psbt(&self) -> crate::Result<PSBT> {
        Ok(psbt_from_base64(&self.psbt)?.1)
    }

    pub fn set_psbt(&mut self, psbt: &PSBT) {
        self.psbt = psbt_to_base64(psbt).1;
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

impl MasterSecretJson {
    pub fn new(
        network: Network,
        mnemonic: &Mnemonic,
        name: &str,
    ) -> crate::Result<MasterSecretJson> {
        let secp = bitcoin::secp256k1::Secp256k1::signing_only();
        let seed = mnemonic.to_seed(None);

        let xprv = ExtendedPrivKey::new_master(network, &seed.0)?;
        let xpub = ExtendedPubKey::from_private(&secp, &xprv);

        Ok(MasterSecretJson {
            mnemonic: Some(mnemonic.clone()),
            xprv,
            xpub,
            dice: None,
            fingerprint: xpub.fingerprint(),
            id: Identifier::new(network, IdKind::MasterSecret, name),
        })
    }

    pub fn from_xprv(xprv: ExtendedPrivKey, name: &str) -> Self {
        let secp = bitcoin::secp256k1::Secp256k1::signing_only();
        let xpub = ExtendedPubKey::from_private(&secp, &xprv);
        MasterSecretJson {
            xprv,
            xpub,
            mnemonic: None,
            dice: None,
            fingerprint: xpub.fingerprint(),
            id: Identifier::new(xprv.network, IdKind::MasterSecret, name),
        }
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
impl_try_into!(WalletSignatureJson);
impl_try_into!(VerifyWalletResult);

#[cfg(test)]
mod tests {
    use crate::WalletJson;

    #[test]
    fn test_cbor_wallet() {
        let wallet: WalletJson = serde_json::from_str(r#"{"id":{"name":"name","kind":"Wallet","network": "bitcoin"},"descriptor":"wsh(multi(3,tpubD6NzVbkrYhZ4XuzR59W1JHQpXcufQVj64NDa4eiALMJxC2xAwpY7wy2J9RVQ7BHDYK3eWrVRsuMUcdwGn9xVBmC9wfpVawpNGLyrdgAhehd/0/*,tpubD6NzVbkrYhZ4WarEBpY5okLrjRQ8sgfoEsxZfprQDEbAjWM585LhNeT9GuSeFRGL7yLheiRgtCQCBb73y21EsLzRfwdrRmfaAT4yUTEKtu7/0/*,tpubD6NzVbkrYhZ4WRwbTYgdGDMxPUzq5WwX8HwnAR6PYB291uUH63pCU1WFV6RRWGyA2Xy8okiFAqfAXEErx1SVh7mKSVQa34hFaa8GcmuEeds/0/*,tpubD6NzVbkrYhZ4YkVm13NDmMPEHEWXHoqGXBPCrtHbB1hE6GoTjdvXEKrtRBMtSe4gQQUSyvU78jgyrK5AfwLewr1cTkkojQbYTuyNtgQFEDb/0/*,tpubD6NzVbkrYhZ4YGeACdA4t1ZjEfJm8ExF818xG2ndsNoT61PwPnotxVQXDLZAZ5ut7t1iHR2FLEYnTzJTN5DGxQTKgwQpt7ftPzRwjugwuYg/0/*))#we4l0t0l","fingerprints":["171f9233","37439b38","ab4343d4","deb8f1ba","214c5f36"],"required_sig":3,"created_at_height":1720730}"#).unwrap();

        let vec_json = serde_json::to_vec(&wallet).unwrap();
        let vec_cbor = serde_cbor::to_vec(&wallet).unwrap();

        assert_eq!(vec_json.len(), 793);
        assert_eq!(vec_cbor.len(), 735);
    }
}
