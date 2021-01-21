use crate::{expand_tilde, Result};
use bitcoin::Network;
use log::debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
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
            IdKind::Wallet => "descriptor.json",          // "wallet.json",
            IdKind::WalletIndexes => "indexes.json",      //"wallet_indexes.json",
            IdKind::WalletSignature => "signature.json",  //"wallet_signature.json",
            IdKind::MasterSecret => "PRIVATE.json",       //"master_secret.json",
            IdKind::DescriptorPublicKey => "public.json", //"descriptor_public_key.json",
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

    pub fn read<T, P>(&self, datadir: P) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug,
        P: AsRef<Path>,
    {
        let path = self.as_path_buf(datadir, false)?;
        debug!("reading {:?}", path);
        let file_content = std::fs::read(&path)
            .map_err(|e| crate::Error::FileNotFoundOrCorrupt(path, e.to_string()))?;
        let data: T = serde_json::from_slice(&file_content)?;
        Ok(data)
    }

    pub fn write<T, P>(&self, datadir: P, value: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Debug,
        P: AsRef<Path>,
    {
        debug!("Identifier::write");
        let path = self.as_path_buf(datadir, true)?;
        let content = serde_json::to_vec_pretty(value)?;
        std::fs::write(&path, &content)?;
        Ok(())
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
        let expected = "\"/bitcoin/keys/a1/PRIVATE.json\""; //TODO master_secret
        let result = format!("{:?}", id.as_path_buf("/", false).unwrap());
        assert_eq!(expected, result);

        let expected = r#"{"kind":"MasterSecret","name":"a1","network":"bitcoin"}"#;
        let result = serde_json::to_value(&id).unwrap();
        assert_eq!(expected, result.to_string());
    }
}
