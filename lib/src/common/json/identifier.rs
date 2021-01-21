use crate::{expand_tilde, Result};
use bitcoin::Network;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum IdKind {
    Wallet,
    WalletIndexes,
    WalletSignature,
    MasterSecret,
    DescriptorPublicKey,
    PSBT,
}

impl IdKind {
    fn dir(&self) -> &str {
        match self {
            IdKind::Wallet | IdKind::WalletIndexes | IdKind::WalletSignature => "wallets",
            IdKind::MasterSecret | IdKind::DescriptorPublicKey => "keys",
            IdKind::PSBT => "psbts",
        }
    }

    fn name(&self) -> &str {
        match self {
            IdKind::Wallet => "wallet.json",
            IdKind::WalletIndexes => "wallet_indexes.json",
            IdKind::WalletSignature => "wallet_signature.json",
            IdKind::MasterSecret => "master_secret.json",
            IdKind::DescriptorPublicKey => "descriptor_public_key.json",
            IdKind::PSBT => "psbt.json",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Identifier {
    network: Network,
    kind: IdKind,
    pub name: String,
}

impl Identifier {
    pub fn new(network: Network, kind: IdKind, name: &str) -> Self {
        Identifier {
            network,
            kind,
            name: name.to_string(),
        }
    }

    pub fn with_kind(&self, new_kind: IdKind) -> Self {
        Identifier {
            network: self.network,
            kind: new_kind,
            name: self.name.clone(),
        }
    }

    pub fn as_path_buf<P: AsRef<Path>>(&self, datadir: P) -> Result<PathBuf> {
        let mut path = expand_tilde(datadir)?;
        path.push(self.network.to_string());
        path.push(self.kind.dir());
        path.push(self.name.to_string());
        path.push(self.kind.name());
        Ok(path)
    }

    pub fn read<T, P>(&self, datadir: P) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug,
        P: AsRef<Path>,
    {
        let path = self.as_path_buf(datadir)?;
        let file_content = std::fs::read(&path)?;
        let data: T = serde_json::from_slice(&file_content)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::json::identifier::{IdKind, Identifier};
    use bitcoin::Network;

    #[test]
    fn test_identifier() {
        let id = Identifier {
            network: Network::Bitcoin,
            kind: IdKind::MasterSecret,
            name: "a1".to_string(),
        };
        let expected = "\"/bitcoin/keys/a1/master_secret.json\"";
        let result = format!("{:?}", id.as_path_buf("/").unwrap());
        assert_eq!(expected, result);

        let expected = r#"{"kind":"MasterSecret","name":"a1","network":"bitcoin"}"#;
        let result = serde_json::to_value(&id).unwrap();
        assert_eq!(expected, result.to_string());
    }
}
