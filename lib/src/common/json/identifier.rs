use crate::Result;
use bitcoin::hashes::core::fmt::Formatter;
use bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum IdKind {
    Wallet,
    Key,
    PSBT,
}

impl Display for IdKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IdKind::Wallet => write!(f, "wallets"),
            IdKind::Key => write!(f, "keys"),
            IdKind::PSBT => write!(f, "psbts"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Identifier {
    pub network: Network,
    pub kind: IdKind,
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

    pub fn as_path_buf(&self, datadir: &str) -> Result<PathBuf> {
        let mut path = PathBuf::from_str(datadir).unwrap();
        path.push(self.network.to_string());
        path.push(self.kind.to_string());
        path.push(self.name.to_string());
        Ok(path)
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
            kind: IdKind::Key,
            name: "a1".to_string(),
        };
        let expected = "\"/bitcoin/keys/a1\"";
        let result = format!("{:?}", id.as_path_buf("/").unwrap());
        assert_eq!(expected, result);

        let expected = r#"{"kind":"Key","name":"a1","network":"bitcoin"}"#;
        let result = serde_json::to_value(&id).unwrap();
        assert_eq!(expected, result.to_string());
    }
}
