use crate::offline::decrypt::{decrypt, EncryptionKey, MaybeEncrypted};
use crate::{expand_tilde, Error, Result, StringEncoding};
use bitcoin::Network;
use core::fmt::Formatter;
use log::debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Kind {
    Wallet,
    WalletIndexes,
    WalletSignature,
    MasterSecret,
    DescriptorPublicKey,
    Psbt,
}

impl Kind {
    pub fn dir(&self) -> &str {
        match self {
            Kind::Wallet | Kind::WalletIndexes | Kind::WalletSignature => "wallets",
            Kind::MasterSecret | Kind::DescriptorPublicKey => "keys",
            Kind::Psbt => "psbts",
        }
    }

    fn name(&self) -> &str {
        match self {
            Kind::Wallet => "wallet.json",
            Kind::WalletIndexes => "wallet_indexes.json",
            Kind::WalletSignature => "wallet_signature.json",
            Kind::MasterSecret => "master_secret.json",
            Kind::DescriptorPublicKey => "descriptor_public_key.json",
            Kind::Psbt => "psbt.json",
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Kind {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Wallet" => Ok(Kind::Wallet),
            "WalletIndexes" => Ok(Kind::WalletIndexes),
            "WalletSignature" => Ok(Kind::WalletSignature),
            "MasterSecret" => Ok(Kind::MasterSecret),
            "DescriptorPublicKey" => Ok(Kind::DescriptorPublicKey),
            "Psbt" => Ok(Kind::Psbt),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("({}) valid values are: wallets, keys, psbts", s),
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Identifier {
    pub kind: Kind,
    pub name: String,
    pub network: Network,
}

pub trait Identifiable {
    fn id(&self) -> &Identifier;
}

pub trait Overwritable {
    fn can_overwrite() -> bool;
}

pub trait WhichKind {
    fn kind() -> Kind;
}

impl Identifier {
    pub fn new(network: Network, kind: Kind, name: &str) -> Self {
        Identifier {
            network,
            kind,
            name: name.to_string(),
        }
    }

    pub fn with_kind(&self, new_kind: Kind) -> Self {
        Identifier {
            network: self.network,
            kind: new_kind,
            name: self.name.clone(),
        }
    }

    pub fn as_path_buf<P: AsRef<Path>>(
        &self,
        datadir: P,
        create_if_missing: bool,
    ) -> Result<PathBuf> {
        let mut path = expand_tilde(datadir)?;
        path.push(self.network.to_string());
        path.push(self.kind.dir());
        path.push(self.name.to_string());
        if create_if_missing && !path.exists() {
            fs::create_dir_all(&path)?;
            debug!("created {:?}", path);
        }
        path.push(self.kind.name());
        Ok(path)
    }

    pub fn read<T, P>(&self, datadir: P, encryption_key: &Option<StringEncoding>) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug,
        P: AsRef<Path>,
    {
        let path = self.as_path_buf(datadir, false)?;
        debug!("reading {:?}", path);

        let data = decrypt(&path, encryption_key)?;

        Ok(data)
    }

    pub fn write<T, P>(
        &self,
        datadir: P,
        value: &T,
        can_overwrite: bool,
        encryption_key: &Option<EncryptionKey>,
    ) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Debug + Clone,
        P: AsRef<Path>,
    {
        let path = self.as_path_buf(datadir, true)?;
        debug!(
            "Identifier::write {:?} can_overwrite:{}",
            path, can_overwrite
        );
        if path.exists() && !can_overwrite {
            return Err(Error::CannotOverwrite(path));
        }

        let plain = MaybeEncrypted::plain(value.clone());
        let data = match encryption_key.as_ref() {
            None => plain,
            Some(encryption_key) => plain.encrypt(encryption_key)?,
        };
        let content = serde_json::to_vec_pretty(&data)?;
        std::fs::write(&path, &content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::tests::rnd_string;
    use crate::{Identifier, Kind};
    use bitcoin::Network;

    impl Identifier {
        pub fn new_test(kind: Kind) -> Self {
            Identifier {
                kind,
                name: rnd_string(),
                network: Network::Testnet,
            }
        }
    }

    #[test]
    fn test_identifier() {
        let id = Identifier {
            network: Network::Bitcoin,
            kind: Kind::MasterSecret,
            name: "a1".to_string(),
        };
        let expected = "\"/bitcoin/keys/a1/master_secret.json\"";
        let result = format!("{:?}", id.as_path_buf("/", false).unwrap());
        assert_eq!(expected, result);

        let expected = r#"{"kind":"MasterSecret","name":"a1","network":"bitcoin"}"#;
        let result = serde_json::to_value(&id).unwrap();
        assert_eq!(expected, result.to_string());
    }
}
