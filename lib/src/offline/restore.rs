use crate::mnemonic::Mnemonic;
use crate::MasterSecretJson;
use crate::{check_compatibility, Context, Result};
use bitcoin::util::bip32::ExtendedPrivKey;
use serde::{Deserialize, Serialize};
use std::io;
use std::str::FromStr;
use structopt::StructOpt;

/// Restore a master key from the secret component
#[derive(StructOpt, Debug, Deserialize, Serialize)]
#[structopt(name = "restore")]
pub struct RestoreOptions {
    /// Name of the key
    #[structopt(short, long)]
    pub key_name: String,

    /// Kind of the secret material
    #[structopt(short, long)]
    pub nature: Nature,

    /// value of the secret component, could be xprv or seed in hex or bech32
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Nature {
    Xprv,
    Mnemonic,
}

impl FromStr for Nature {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "xprv" => Ok(Nature::Xprv),
            "mnemonic" => Ok(Nature::Mnemonic),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("({}) valid values are: xprv, mnemonic", s),
            )),
        }
    }
}

impl Context {
    pub fn restore(&self, opt: &RestoreOptions) -> Result<MasterSecretJson> {
        let master_key = match opt.nature {
            Nature::Xprv => {
                let key = ExtendedPrivKey::from_str(&opt.value)?;
                check_compatibility(key.network, self.network)?;
                MasterSecretJson::from_xprv(key, &opt.key_name, self.network)
            }
            Nature::Mnemonic => {
                let mnemonic = Mnemonic::from_str(&opt.value)?;
                MasterSecretJson::new(self.network, &mnemonic, &opt.key_name)?
            }
        };
        self.write_keys(&master_key)?;

        Ok(master_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::context::tests::TestContext;
    use crate::list::ListOptions;
    use crate::offline::random::RandomOptions;
    use crate::offline::restore::{Nature, RestoreOptions};
    use crate::Kind;
    use bitcoin::Network;

    #[test]
    fn test_restore() {
        let context = TestContext::new();
        let key_name_random = "test_restore_random".to_string();
        let rand_opts = RandomOptions {
            key_name: key_name_random,
        };
        let name_counter = 0;

        let key_orig = context.create_key(&rand_opts).unwrap();

        let (key_name, name_counter) = (format!("{}", name_counter), name_counter + 1);
        let restore_opts = RestoreOptions {
            key_name: key_name.clone(),
            nature: Nature::Xprv,
            value: key_orig.xprv.to_string(),
        };
        let key_restored = context.restore(&restore_opts).unwrap();
        assert_eq!(key_orig.xprv, key_restored.xprv);
        assert_eq!(key_orig.xpub, key_restored.xpub);
        assert_ne!(key_orig.mnemonic, key_restored.mnemonic);
        let list_options = ListOptions {
            kind: Kind::MasterSecret,
            verify_wallets_signatures: false,
        };
        let list = context.list(&list_options).unwrap();
        assert!(list.master_secrets.iter().any(|a| &a.id.name == &key_name));

        let (key_name, name_counter) = (format!("{}", name_counter), name_counter + 1);
        let restore_opts = RestoreOptions {
            key_name,
            nature: Nature::Mnemonic,
            value: key_orig.mnemonic.as_ref().unwrap().to_string(),
        };
        let key_restored = context.restore(&restore_opts).unwrap();
        assert_eq!(key_orig.xprv, key_restored.xprv);
        assert_eq!(key_orig.xpub, key_restored.xpub);
        assert_eq!(&key_orig.mnemonic, &key_restored.mnemonic);

        // TODO add restore mnemonic

        let (key_name, name_counter) = (format!("{}", name_counter), name_counter + 1);
        let restore_opts = RestoreOptions {
            key_name,
            nature: Nature::Xprv,
            value: "X".to_string(),
        };
        let result = context.restore(&restore_opts);
        assert!(result.is_err());

        let (key_name, _name_counter) = (format!("{}", name_counter), name_counter + 1);
        let restore_opts = RestoreOptions {
            key_name: key_name.clone(),
            nature: Nature::Xprv,
            value: key_orig.xpub.to_string(),
        };
        let result = context.restore(&restore_opts);
        assert!(result.is_err());

        let regtest_context = TestContext::with_network(Network::Regtest);
        let key_orig = regtest_context.create_key(&rand_opts).unwrap();
        let restore_opts = RestoreOptions {
            key_name: key_name.clone(),
            nature: Nature::Xprv,
            value: key_orig.xprv.to_string(),
        };
        let _ = regtest_context.restore(&restore_opts).unwrap();
        let list = regtest_context.list(&list_options).unwrap();
        assert!(list.master_secrets.iter().any(|a| &a.id.name == &key_name));
    }
}
