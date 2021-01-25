use crate::common::json::identifier::{Identifiable, Identifier, Overwriteable, WhichKind};
use crate::*;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::Network;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    pub network: bitcoin::Network,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub firma_datadir: String, //TODO rename datadir

                               //TODO phantom data for offline and online
                               //TODO maybe encryption key should belong here
}

#[derive(StructOpt, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DaemonOpts {
    /// Bitcoin node rpc url
    #[structopt(long)]
    pub url: String,

    /// Bitcoin node cookie file
    #[structopt(long)]
    pub cookie_file: PathBuf,
}

impl DaemonOpts {
    /// creates RPC client to bitcoin node, with the specified wallet name.
    /// It also checks the `network` parameter is the same as the connecting node
    pub fn make_client(&self, wallet_name: Option<String>, network: Network) -> Result<Client> {
        let url = match wallet_name {
            Some(wallet) => format!("{}/wallet/{}", self.url, wallet),
            None => self.url.to_string(),
        };
        debug!("creating client with url {}", url);
        let client = Client::new(url, Auth::CookieFile(self.cookie_file.clone()))?;
        let genesis = client.get_block_hash(0)?;
        if genesis != genesis_block(network).block_hash() {
            return Err(Error::IncompatibleNetworks);
        }
        Ok(client)
    }
}

impl Context {
    pub fn base(&self) -> Result<PathBuf> {
        let mut path = expand_tilde(&self.firma_datadir)?;
        path.push(self.network.to_string());
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    pub fn read<T>(&self, name: &str) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug + WhichKind,
    {
        Ok(Identifier::new(self.network, T::kind(), name).read(&self.firma_datadir)?)
    }

    pub fn write<T>(&self, value: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Debug + Identifiable + Overwriteable,
    {
        value
            .id()
            .write(&self.firma_datadir, value, T::can_overwrite())
    }

    pub fn write_keys(&self, master_key: &MasterSecretJson) -> Result<()> {
        self.write(master_key)?;
        let public: PublicMasterKey = master_key.clone().into();
        self.write(&public)
    }

    fn daemon_opts_path(&self) -> Result<PathBuf> {
        let mut path = self.base()?;
        path.push("daemon_opts.json");
        Ok(path)
    }

    pub fn read_daemon_opts(&self) -> Result<DaemonOpts> {
        let path = self.daemon_opts_path()?;
        debug!("reading daemon_opts from {:?}", path);
        let bytes = std::fs::read(&path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn write_daemon_opts(&self, daemon_opts: DaemonOpts) -> Result<DaemonOpts> {
        let path = self.daemon_opts_path()?;
        debug!("writing daemon_opts in {:?}", path);
        let bytes = serde_json::to_vec(&daemon_opts)?;
        std::fs::write(&path, &bytes)
            .map_err(|e| crate::Error::FileNotFoundOrCorrupt(path, e.to_string()))?;
        Ok(daemon_opts)
    }
    pub fn make_client(&self, wallet_name: &str) -> Result<Client> {
        let client = self
            .read_daemon_opts()?
            .make_client(Some(wallet_name.to_string()), self.network)?;
        load_if_unloaded(&client, wallet_name)?;
        Ok(client)
    }
}

pub fn load_if_unloaded(client: &Client, wallet_name: &str) -> Result<()> {
    match client.load_wallet(wallet_name) {
        Ok(_) => info!("wallet {} loaded", wallet_name),
        Err(e) => {
            if e.to_string().contains("not found") {
                return Err(format!("{} not found in the bitcoin node", wallet_name).into());
            } else {
                debug!("wallet {} already loaded", wallet_name);
            }
        }
    }
    Ok(())
}

// from https://stackoverflow.com/questions/54267608/expand-tilde-in-rust-path-idiomatically
pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Result<PathBuf> {
    let p = path_user_input.as_ref();
    if p.starts_with("~") {
        let mut home_dir = dirs_next::home_dir().ok_or(Error::CannotRetrieveHomeDir)?;
        if p == Path::new("~") {
            Ok(home_dir)
        } else if home_dir == Path::new("/").to_path_buf() {
            // Corner case: `home_dir` root directory;
            // don't prepend extra `/`, just drop the tilde.
            Ok(p.strip_prefix("~")?.to_path_buf())
        } else {
            home_dir.push(p.strip_prefix("~/")?);
            Ok(home_dir)
        }
    } else {
        Ok(p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use crate::offline::random::RandomOptions;
    use crate::{Context, MasterSecretJson, PublicMasterKey};
    use bitcoin::Network;
    use std::ops::Deref;
    use tempfile::TempDir;

    struct TestContext {
        context: Context,
        #[allow(unused)]
        datadir: TempDir, // must be here so directory isnt't removed before dropping the object
    }

    impl TestContext {
        fn new() -> Self {
            let datadir = TempDir::new().unwrap();
            let firma_datadir = format!("{}/", datadir.path().display());
            let network = Network::Testnet;
            TestContext {
                context: Context {
                    network,
                    firma_datadir,
                },
                datadir,
            }
        }
    }

    impl Deref for TestContext {
        type Target = Context;

        fn deref(&self) -> &Self::Target {
            &self.context
        }
    }

    #[test]
    fn test_write_keys() {
        let context = TestContext::new();
        let key_name = "a";
        let key = context
            .create_key(&RandomOptions {
                key_name: key_name.to_string(),
                encryption_key: None,
            })
            .unwrap();
        assert!(
            context.write_keys(&key).is_err(),
            "can overwrite key material"
        );
        let key_read: MasterSecretJson = context.read(&key_name).unwrap();
        assert_eq!(key, key_read);
        let _: PublicMasterKey = context.read(&key_name).unwrap();
    }
}
