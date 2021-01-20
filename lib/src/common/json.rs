use crate::common::mnemonic::Mnemonic;
use crate::offline::sign::get_psbt_name;
use crate::{psbt_from_base64, psbt_to_base64, PSBT, Result};
use bitcoin::bech32::FromBase32;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::util::psbt::{raw, Map};
use bitcoin::{bech32, Address, Amount, Network, OutPoint, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::WalletCreateFundedPsbtResult;
use miniscript::descriptor::DescriptorXKey;
use miniscript::DescriptorPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::convert::TryInto;
use std::path::PathBuf;
use std::str::FromStr;



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MnemonicOrXpriv {
    Mnemonic(Mnemonic, Network),
    Xpriv(ExtendedPrivKey),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SecretMasterKey {
    /// The name of this key
    pub name: String,

    /// The secret material of this key in mnemonic form or as extended private key
    pub secret: MnemonicOrXpriv,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Dice material if this key has been created with dice
    pub dice: Option<Dice>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DescriptorPublicKeyJson {
    /// ToString of [miniscript::descriptor::DescriptorPublicKey]
    /// Example: `[28645006/48'/1'/0'/2']tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s/0/*`
    /// TODO use DescriptorPublicKey when implement Serialize
    pub desc_pub_key: String,
}

//TODO save this data in a file? Must be encrypted, encrypt everything
//TODO use coldcard algo https://coldcardwallet.com/docs/rolls.py
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dice {
    pub launches: String,
    pub faces: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterKeyOutput {
    pub master_secret: SecretMasterKey,
    pub desc_public: DescriptorPublicKeyJson,
    pub private_file: PathBuf,
    pub public_file: PathBuf,
    pub public_qr_files: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PsbtJson {
    pub name: String,
    pub psbt: String,
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
pub struct WalletJson {
    pub name: String,
    pub descriptor: String,
    pub fingerprints: BTreeSet<Fingerprint>, // TODO derive from descriptor, or include in descriptor?
    pub required_sig: usize,                 // TODO derive from descriptor?
    pub created_at_height: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletSignature {
    pub xpub: ExtendedPubKey,
    pub address: Address,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletIndexes {
    pub main: u32,
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
    pub signature: Option<WalletSignature>,
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
    pub signature: WalletSignature,
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

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        Ok(match self {
            StringEncoding::Base64(s) => base64::decode(s)?,
            StringEncoding::Hex(s) => hex::decode(s)?,
            StringEncoding::Bech32(s) => {
                let (_, vec_u5) = bech32::decode(s)?;
                Vec::<u8>::from_base32(&vec_u5)?
            }
        })
    }

    pub fn get_exactly_32(&self) -> Result<[u8; 32]> {
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

pub fn psbt_from_rpc(psbt: &WalletCreateFundedPsbtResult, name: &str) -> Result<PSBT> {
    let (_, mut psbt_with_name) = psbt_from_base64(&psbt.psbt)?;

    let pair = raw::Pair {
        key: get_name_key(),
        value: name.as_bytes().to_vec(),
    };
    psbt_with_name.global.insert_pair(pair)?;
    Ok(psbt_with_name)
}

impl From<&PSBT> for PsbtJson {
    fn from(psbt: &PSBT) -> Self {
        let (_, base64) = psbt_to_base64(psbt);
        let name = get_psbt_name(psbt).expect("PSBT without name"); //TODO
        PsbtJson { psbt: base64, name }
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

impl MnemonicOrXpriv {
    pub fn network(&self) -> &Network {
        match self {
            MnemonicOrXpriv::Mnemonic(_,network) => network,
            MnemonicOrXpriv::Xpriv(xpriv ) => &xpriv.network
        }
    }
    pub fn xprv(&self) -> &ExtendedPrivKey {
        match self {
            MnemonicOrXpriv::Mnemonic(mnemonic,network) => {
                let seed = mnemonic.to_seed(None);
                let xprv = ExtendedPrivKey::new_master(*network, &seed.0).unwrap(); //TODO return Result
                &xprv
            },
            MnemonicOrXpriv::Xpriv(xpriv ) => xpriv
        }
    }
}

impl Dice {
    pub fn as_mnemonic(&self) -> Mnemonic {
        //TODO implement as coldcard?
        unimplemented!();
    }
}

impl SecretMasterKey {
    pub fn from_mnemonic(
        network: Network,
        mnemonic: Mnemonic,
        name: &str,
    ) -> Self {
        SecretMasterKey {
            secret: MnemonicOrXpriv::Mnemonic(mnemonic, network),
            name: name.to_string(),
            dice: None,
        }
    }

    pub fn from_xprv(xprv: ExtendedPrivKey, name: &str) -> Self {
        SecretMasterKey {
            secret: MnemonicOrXpriv::Xpriv(xprv),
            name: name.to_string(),
            dice: None,
        }
    }

    pub fn from_dice(network: Network, dice: Dice, name: &str) -> Self {
        let mnemonic = dice.as_mnemonic();
        SecretMasterKey {
            secret: MnemonicOrXpriv::Mnemonic(mnemonic, network),
            name: name.to_string(),
            dice: Some(dice),
        }
    }

    pub fn as_desc_pub_key(&self, custom_origin: &Option<DerivationPath>) -> Result<DescriptorPublicKeyJson> {
        let secp = bitcoin::secp256k1::Secp256k1::signing_only();

        let network = self.secret.network();
        let path = custom_origin.unwrap_or_else(|| {
            let n = match network {
                Network::Bitcoin => "0",
                Network::Testnet => "1",
                Network::Regtest => "2", // bip48 skip this
            };
            DerivationPath::from_str(&format!("m/48'/{}'/0'/2'", n)).unwrap() // safe
        });
        let xprv_derived = self.secret.xprv().derive_priv(&secp, &path)?;
        let xpub = ExtendedPubKey::from_private(&secp, &xprv_derived);
        let desc_pub_key = DescriptorPublicKey::XPub(DescriptorXKey {
            origin: Some((self.secret.xprv().fingerprint(&secp), path)),
            xkey: xpub,
            derivation_path: DerivationPath::from_str("m/0")?,
            is_wildcard: true,
        });
        Ok(DescriptorPublicKeyJson{desc_pub_key: desc_pub_key.to_string()})
    }
}

macro_rules! impl_try_into {
    ( $for:ty ) => {
        impl TryInto<Value> for $for {
            type Error = crate::Error;

            fn try_into(self) -> std::result::Result<Value, Self::Error> {
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
impl_try_into!(WalletSignature);
impl_try_into!(VerifyWalletResult);

#[cfg(test)]
mod tests {
    use crate::common::mnemonic::Mnemonic;
    use crate::{SecretMasterKey, WalletJson};
    use bitcoin::Network;
    use std::str::FromStr;

    #[test]
    fn test_cbor_wallet() {
        let wallet: WalletJson = serde_json::from_str("{\"name\":\"w3of5\",\"descriptor\":\"wsh(multi(3,tpubD6NzVbkrYhZ4XuzR59W1JHQpXcufQVj64NDa4eiALMJxC2xAwpY7wy2J9RVQ7BHDYK3eWrVRsuMUcdwGn9xVBmC9wfpVawpNGLyrdgAhehd/0/*,tpubD6NzVbkrYhZ4WarEBpY5okLrjRQ8sgfoEsxZfprQDEbAjWM585LhNeT9GuSeFRGL7yLheiRgtCQCBb73y21EsLzRfwdrRmfaAT4yUTEKtu7/0/*,tpubD6NzVbkrYhZ4WRwbTYgdGDMxPUzq5WwX8HwnAR6PYB291uUH63pCU1WFV6RRWGyA2Xy8okiFAqfAXEErx1SVh7mKSVQa34hFaa8GcmuEeds/0/*,tpubD6NzVbkrYhZ4YkVm13NDmMPEHEWXHoqGXBPCrtHbB1hE6GoTjdvXEKrtRBMtSe4gQQUSyvU78jgyrK5AfwLewr1cTkkojQbYTuyNtgQFEDb/0/*,tpubD6NzVbkrYhZ4YGeACdA4t1ZjEfJm8ExF818xG2ndsNoT61PwPnotxVQXDLZAZ5ut7t1iHR2FLEYnTzJTN5DGxQTKgwQpt7ftPzRwjugwuYg/0/*))#we4l0t0l\",\"fingerprints\":[\"171f9233\",\"37439b38\",\"ab4343d4\",\"deb8f1ba\",\"214c5f36\"],\"required_sig\":3,\"created_at_height\":1720730}").unwrap();

        let vec_json = serde_json::to_vec(&wallet).unwrap();
        let vec_cbor = serde_cbor::to_vec(&wallet).unwrap();

        assert_eq!(vec_json.len(), 751);
        assert_eq!(vec_cbor.len(), 704);
    }

    #[test]
    fn test_new_private_json() {
        let key_json = SecretMasterKey::from_mnemonic(
            Network::Testnet,
            Mnemonic::from_str(
                "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            )
            .unwrap(),
            "ciao",
        );

        assert_eq!(key_json.as_desc_pub_key(&None).unwrap().desc_pub_key, "");
    }
}
