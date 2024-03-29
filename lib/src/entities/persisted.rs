// https://dreampuf.github.io/GraphvizOnline/#digraph%20G%20%7B%0A%20%20%22.firma%22%20-%3E%20%22%5Bnetwork%5D%22%0A%20%20%0A%20%20%22%5Bnetwork%5D%22%20-%3E%20wallets%0A%20%20%22%5Bnetwork%5D%22%20-%3E%20keys%0A%20%20%22%5Bnetwork%5D%22%20-%3E%20psbts%0A%20%20%22%5Bnetwork%5D%22%20-%3E%20%22daemon_opts%22%20%0A%20%20%0A%20%20keys%20-%3E%20%22%5Bkey%20name%5D%22%0A%20%20%22master_secret%22%20%5Bshape%3DSquare%5D%0A%20%20%22descriptor_public_key%22%20%5Bshape%3DSquare%5D%0A%20%20%22%5Bkey%20name%5D%22%20-%3E%20%22master_secret%22%20%0A%20%20%22%5Bkey%20name%5D%22%20-%3E%20%22descriptor_public_key%22%20%0A%20%20%0A%20%20wallets%20-%3E%20%22%5Bwallet%20name%5D%22%0A%20%20%22wallet%22%20%5Bshape%3DSquare%5D%0A%20%20%22wallet_indexes%22%20%5Bshape%3DSquare%5D%0A%20%20%22daemon_opts%22%20%5Bshape%3DSquare%5D%0A%20%20%22wallet_signature%22%20%5Bshape%3DSquare%5D%0A%20%20%22%5Bwallet%20name%5D%22%20-%3E%20%22wallet%22%20%0A%20%20%22%5Bwallet%20name%5D%22%20-%3E%20%22wallet_indexes%22%20%0A%20%20%22%5Bwallet%20name%5D%22%20-%3E%20%22wallet_signature%22%20%0A%20%20%0A%20%20psbts%20-%3E%20%22%5Bpsbt%20name%5D%22%0A%20%20%22psbt%22%20%5Bshape%3DSquare%5D%0A%20%20%22%5Bpsbt%20name%5D%22%20-%3E%20%22psbt%22%20%0A%7D

use crate::mnemonic::Mnemonic;
use crate::offline::descriptor::{parse_descriptor_with_checksum, ExtendedDescriptor};
use crate::offline::sign::get_psbt_name;
use crate::offline::sign_wallet::WALLET_SIGN_DERIVATION;
use crate::{
    check_compatibility, psbt_from_base64, psbt_to_base64, BitcoinPsbt, Error, Identifier, Kind,
    Result,
};
use bitcoin::secp256k1::{Secp256k1, Signing};
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::{secp256k1, Network};
use log::debug;
use miniscript::descriptor::{DescriptorXKey, Wildcard};
use miniscript::{Descriptor, ForEachKey};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Wallet {
    pub id: Identifier,
    pub descriptor: String,
    pub created_at_height: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletIndexes {
    pub id: Identifier,
    pub main: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletSignature {
    pub id: Identifier,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MasterSecret {
    pub id: Identifier,
    pub key: ExtendedPrivKey,
    pub network: Network, // ExtendedPrivKey is not enough since regtest is the same as testnet
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<Mnemonic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dice: Option<Dice>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DescriptorPublicKey {
    //TODO make it DescriptorPublicKeyJson
    pub id: Identifier,
    /// ToString of [miniscript::descriptor::DescriptorPublicKey]
    /// Example: `[28645006/48'/1'/0'/2']tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s/0/*`
    /// TODO use DescriptorPublicKey when implement Serialize
    pub desc_pub_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Psbt {
    pub id: Identifier,
    /// PSBT serialized with base64
    pub psbt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dice {
    pub launches: String,
    pub faces: u32,
    pub value: String,
}

impl Wallet {
    pub fn extract_desc_pub_keys(&self) -> Result<Vec<miniscript::DescriptorPublicKey>> {
        let mut desc_pub_keys = vec![];
        let end = self.descriptor.find('#').unwrap_or(self.descriptor.len());
        let descriptor: miniscript::Descriptor<miniscript::DescriptorPublicKey> =
            self.descriptor[..end].parse().unwrap();
        if let Descriptor::Wsh(miniscript) = descriptor {
            miniscript.for_each_key(|k| {
                desc_pub_keys.push(k.as_key().clone());
                true
            });
        }
        Ok(desc_pub_keys)
    }
    pub fn extract_wallet_sign_keys(&self) -> Result<Vec<bitcoin::PublicKey>> {
        let secp = Secp256k1::verification_only();
        let mut keys = vec![];
        for k in self.extract_desc_pub_keys()? {
            debug!("extract_wallet_sign_keys descriptor_pub_key:{} ", k);
            let key = k.derive(WALLET_SIGN_DERIVATION).derive_public_key(&secp)?;
            debug!(
                "extract_wallet_sign_keys public_key:{} WALLET_SIGN_DERIVATION:{}",
                key, WALLET_SIGN_DERIVATION
            );
            keys.push(key);
        }
        Ok(keys)
    }
    pub fn fingerprints(&self) -> Vec<Fingerprint> {
        let mut result = vec![];
        if let Ok(v) = self.extract_desc_pub_keys() {
            for k in v {
                if let miniscript::DescriptorPublicKey::XPub(x) = k {
                    if let Some(f) = x.origin {
                        result.push(f.0);
                    }
                }
            }
        }
        result
    }
    pub fn descriptor(&self) -> Result<ExtendedDescriptor> {
        parse_descriptor_with_checksum(&self.descriptor)
    }
}

impl DescriptorPublicKey {
    pub fn key(&self) -> Result<miniscript::DescriptorPublicKey> {
        Ok(self.desc_pub_key.parse()?)
    }
    fn xkey(&self) -> Result<DescriptorXKey<ExtendedPubKey>> {
        if let Ok(miniscript::DescriptorPublicKey::XPub(x)) = self.key() {
            return Ok(x);
        }
        Err(Error::WrongKeyType)
    }
    pub fn origin_path(&self) -> Result<DerivationPath> {
        Ok(self.xkey()?.origin.ok_or(Error::WrongKeyType)?.1)
    }
    pub fn xpub(&self) -> Result<ExtendedPubKey> {
        Ok(self.xkey()?.xkey)
    }
}

impl MasterSecret {
    pub fn from_xprv(network: Network, key: ExtendedPrivKey, name: &str) -> Result<Self> {
        check_compatibility(network, key.network)?;
        let fingerprint = key.fingerprint(&Secp256k1::signing_only()).to_string();

        Ok(MasterSecret {
            key,
            network,
            fingerprint,
            mnemonic: None,
            dice: None,
            id: Identifier::new(network, Kind::MasterSecret, name),
        })
    }

    pub fn new(network: Network, mnemonic: Mnemonic, name: &str) -> Result<Self> {
        let seed = mnemonic.to_seed(None);
        let key = ExtendedPrivKey::new_master(network, &seed.0)?;
        let fingerprint = key.fingerprint(&Secp256k1::signing_only()).to_string();

        Ok(MasterSecret {
            key,
            network,
            fingerprint,
            mnemonic: Some(mnemonic),
            dice: None,
            id: Identifier::new(network, Kind::MasterSecret, name),
        })
    }

    fn path(&self) -> DerivationPath {
        //TODO copay multisig derivation, not sure
        let n = match self.network {
            Network::Bitcoin => "0",
            Network::Testnet => "1",
            Network::Regtest => "2",
            Network::Signet => "3",
        };
        // m / 48' / coin_type' / account' / script_type' / change / address_index
        DerivationPath::from_str(&format!("m/48'/{}'/0'/2'", n)).unwrap()
    }

    pub fn as_desc_prv_key<T: Signing>(&self, secp: &Secp256k1<T>) -> Result<ExtendedPrivKey> {
        debug!("Using path: {}", self.path());
        Ok(self.key.derive_priv(secp, &self.path())?)
    }

    pub fn as_wallet_sign_prv_key<T: Signing>(
        &self,
        secp: &Secp256k1<T>,
    ) -> Result<ExtendedPrivKey> {
        let k = self.as_desc_prv_key(secp)?;
        Ok(k.derive_priv(
            secp,
            &DerivationPath::from_str(&format!("m/0/{}", WALLET_SIGN_DERIVATION))?,
        )?)
    }

    pub fn as_wallet_sign_pub_key<T: Signing>(
        &self,
        secp: &Secp256k1<T>,
    ) -> Result<bitcoin::PublicKey> {
        let k = self.as_wallet_sign_prv_key(secp)?;
        let xpub = ExtendedPubKey::from_private(secp, &k);
        Ok(xpub.public_key)
    }

    /// returns the public part of the key, it is an expensive method cause it's initializing a
    /// secp context
    pub fn as_desc_pub_key(&self) -> Result<DescriptorPublicKey> {
        let secp = Secp256k1::signing_only();
        let xprv_derived = self.as_desc_prv_key(&secp)?;
        let xpub = ExtendedPubKey::from_private(&secp, &xprv_derived);
        let desc_pub_key = miniscript::DescriptorPublicKey::XPub(DescriptorXKey {
            origin: Some((self.key.fingerprint(&secp), self.path())),
            xkey: xpub,
            derivation_path: DerivationPath::from_str("m/0")?,
            wildcard: Wildcard::Unhardened,
        });
        let id = self.id.with_kind(Kind::DescriptorPublicKey);
        Ok(DescriptorPublicKey {
            id,
            desc_pub_key: desc_pub_key.to_string(),
        })
    }

    pub fn fingerprint<S: secp256k1::Signing>(&self, secp: &Secp256k1<S>) -> Fingerprint {
        self.key.fingerprint(secp)
    }
}

impl Psbt {
    pub fn psbt(&self) -> Result<BitcoinPsbt> {
        Ok(psbt_from_base64(&self.psbt)?.1)
    }

    pub fn set_psbt(&mut self, psbt: &BitcoinPsbt) {
        self.psbt = psbt_to_base64(psbt).1;
    }
}

impl From<(&BitcoinPsbt, Network)> for Psbt {
    fn from(psbt_and_network: (&BitcoinPsbt, Network)) -> Self {
        let (psbt, network) = psbt_and_network;
        let (_, base64) = psbt_to_base64(psbt);
        let name = get_psbt_name(psbt).expect("PSBT without name"); //TODO
        Psbt {
            psbt: base64,
            id: Identifier::new(network, Kind::Psbt, &name),
        }
    }
}
