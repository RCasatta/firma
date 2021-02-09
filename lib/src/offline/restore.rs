use crate::mnemonic::Mnemonic;
use crate::Result;
use crate::{MasterSecret, OfflineContext};
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

impl OfflineContext {
    pub fn restore(&self, opt: &RestoreOptions) -> Result<MasterSecret> {
        let master_key = match opt.nature {
            Nature::Xprv => {
                let key = ExtendedPrivKey::from_str(&opt.value)?;
                MasterSecret::new(self.network, key, &opt.key_name)?
            }
            Nature::Mnemonic => {
                let mnemonic = Mnemonic::from_str(&opt.value)?;
                MasterSecret::from_mnemonic(self.network, &mnemonic, &opt.key_name)?
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
        let context = TestContext::default();
        let rand_opts = RandomOptions::new_random();
        let key_name = rand_opts.key_name.as_str();
        let key_orig = context.create_key(&rand_opts).unwrap();

        let restore_opts = RestoreOptions {
            key_name: "restored".to_string(),
            nature: Nature::Xprv,
            value: key_orig.key.to_string(),
        };
        let key_restored = context.restore(&restore_opts).unwrap();
        assert_eq!(key_orig.key, key_restored.key);
        let list_options = ListOptions {
            kind: Kind::MasterSecret,
        };
        let list = context.list(&list_options).unwrap();
        assert!(list.master_secrets.iter().any(|a| &a.id.name == key_name));

        /*
        // TODO add restore mnemonic
        let (key_name, name_counter) = (format!("{}", name_counter), name_counter + 1);
        let restore_opts = RestoreOptions {
            key_name,
            nature: Nature::Mnemonic,
            value: key_orig.mnemonic.as_ref().unwrap().to_string(),
        };
        let key_restored = context.restore(&restore_opts).unwrap();
        assert_eq!(key_orig.xprv(), key_restored.xprv());
        */

        let restore_opts = RestoreOptions {
            key_name: "err".to_string(),
            nature: Nature::Xprv,
            value: "X".to_string(),
        };
        let result = context.restore(&restore_opts);
        assert!(result.is_err());

        let restore_opts = RestoreOptions {
            key_name: "foo".to_string(),
            nature: Nature::Xprv,
            value: key_orig
                .as_desc_pub_key()
                .unwrap()
                .xpub()
                .unwrap()
                .to_string(),
        };
        let result = context.restore(&restore_opts);
        assert!(result.is_err());

        let regtest_context = TestContext::with_network(Network::Regtest);
        let key_orig = regtest_context.create_key(&rand_opts).unwrap();
        let restore_opts = RestoreOptions {
            key_name: "bar".to_string(),
            nature: Nature::Xprv,
            value: key_orig.key.to_string(),
        };
        let _ = regtest_context.restore(&restore_opts).unwrap();
        let list = regtest_context.list(&list_options).unwrap();
        assert!(list.master_secrets.iter().any(|a| &a.id.name == &key_name));
    }
}
