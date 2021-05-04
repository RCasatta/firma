use crate::common::entities::identifier::{Identifiable, Identifier, Overwritable, WhichKind};
use crate::list::ListOptions;
use crate::offline::decrypt::EncryptionKey;
use crate::offline::sign::find_or_create;
use crate::offline::sign::get_psbt_name;
use crate::*;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::consensus::deserialize;
use bitcoin::Network;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use core::ops::DerefMut;
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Network (bitcoin, testnet, regtest, signet)
    #[structopt(short, long, default_value = "testnet")]
    pub network: bitcoin::Network,

    /// Directory where wallet info are saved
    #[structopt(short, long, default_value = "~/.firma/")]
    pub datadir: String,

    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
    //TODO add secp context all here
}

#[derive(StructOpt, Debug, Clone)]
pub struct OnlineContext {
    #[structopt(flatten)]
    context: Context,
}

#[derive(StructOpt, Debug, Clone)]
pub struct OfflineContext {
    #[structopt(flatten)]
    context: Context,
}

macro_rules! impl_context {
    ( $for:ty ) => {
        impl Deref for $for {
            type Target = Context;

            fn deref(&self) -> &Self::Target {
                &self.context
            }
        }

        impl DerefMut for $for {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.context
            }
        }

        impl From<Context> for $for {
            fn from(context: Context) -> Self {
                Self { context }
            }
        }
    };
}

impl_context!(OnlineContext);
impl_context!(OfflineContext);

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
        let node_genesis = client.get_block_hash(0)?;
        let firma_genesis = genesis_block(network).block_hash();
        if node_genesis != firma_genesis {
            return Err(Error::IncompatibleGenesis {
                node: node_genesis,
                firma: firma_genesis,
            });
        }
        Ok(client)
    }
}

impl Context {
    pub fn base(&self) -> Result<PathBuf> {
        let mut path = expand_tilde(&self.datadir)?;
        path.push(self.network.to_string());
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    pub fn encryption_key(&self) -> Option<EncryptionKey> {
        self.encryption_key
            .as_ref()
            .map(|k| k.get_exactly_32().unwrap())
    }

    pub fn read<T>(&self, name: &str) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug + WhichKind,
    {
        Identifier::new(self.network, T::kind(), name).read(&self.datadir, &self.encryption_key)
    }

    pub fn write<T>(&self, value: &T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Debug + Clone + Identifiable + Overwritable,
    {
        value.id().write(
            &self.datadir,
            value,
            T::can_overwrite(),
            &self.encryption_key(),
        )
    }

    pub fn write_keys(&self, master_key: &MasterSecret) -> Result<()> {
        self.write(master_key)?;
        let public: DescriptorPublicKey = master_key.as_desc_pub_key()?;
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
        let bytes = std::fs::read(&path)
            .map_err(|e| crate::Error::FileNotFoundOrCorrupt(path.clone(), e.to_string()))?;
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
        let opts = self.read_daemon_opts()?;
        let default_client = opts.make_client(None, self.network)?;
        let wallet_name_string = wallet_name.to_string();
        if !default_client.list_wallets()?.contains(&wallet_name_string) {
            return Err(Error::WalletNotExistsInNode(wallet_name_string));
        }
        let client = opts.make_client(Some(wallet_name_string), self.network)?;
        load_if_unloaded(&client, wallet_name)?;
        Ok(client)
    }

    pub fn read_desc_pub_keys_from_names(
        &self,
        names: &[String],
    ) -> Result<Vec<miniscript::DescriptorPublicKey>> {
        let mut result = vec![];
        for name in names {
            let id = Identifier::new(self.network, Kind::DescriptorPublicKey, name);
            let json: DescriptorPublicKey = id.read(&self.datadir, &self.encryption_key)?;
            result.push(json.key()?);
        }
        Ok(result)
    }

    pub fn read_encryption_key(&mut self) -> Result<()> {
        // read encryption key from stdin and initialize encryption_key field
        let mut buffer = vec![];
        std::io::stdin().read_to_end(&mut buffer)?;
        let encoded = StringEncoding::new_base64(&buffer);
        self.encryption_key = Some(encoded);
        Ok(())
    }

    pub fn save_psbt_options(&self, opt: &SavePsbtOptions) -> Result<()> {
        info!("save_psbt_options {:?}", opt);
        let bytes = opt
            .psbt
            .as_bytes()
            .map_err(|_| Error::PsbtBadStringEncoding(opt.psbt.kind()))?;
        let mut psbt: BitcoinPsbt = deserialize(&bytes).map_err(Error::PsbtCannotDeserialize)?;

        self.save_psbt(&mut psbt)?;
        Ok(())
    }

    /// psbts_dir is general psbts dir, name is extracted from PSBT
    /// if file exists a PSBT merge will be attempted
    pub fn save_psbt(&self, psbt: &mut BitcoinPsbt) -> Result<String> {
        debug!("save_psbt");

        let name = match get_psbt_name(psbt) {
            Some(name) => name,
            None => {
                let opt = ListOptions { kind: Kind::Psbt };
                let psbts = self.list(&opt)?.psbts;
                find_or_create(psbt, psbts)?
            }
        };

        debug!("psbt_name: {}", name);
        let id = Identifier::new(self.network, Kind::Psbt, &name);
        if let Ok(existing_psbt) = self.read::<Psbt>(&name) {
            info!("old psbt exist, merging together");
            let existing_psbt = existing_psbt.psbt()?;
            psbt.merge(existing_psbt.clone())?;
            if psbt == &existing_psbt {
                return Err(Error::PsbtNotChangedAfterMerge);
            }
        }
        let psbt = psbt_to_base64(&psbt).1;
        let psbt_json = Psbt { id, psbt };
        self.write(&psbt_json)?;
        debug!("finish");
        Ok(name)
    }
}

pub fn load_if_unloaded(client: &Client, wallet_name: &str) -> Result<()> {
    match client.load_wallet(wallet_name) {
        Ok(_) => info!("wallet {} loaded", wallet_name),
        Err(e) => {
            debug!("load_if_unloaded error {:?}", e);
            if e.to_string().contains("not found") {
                return Err(format!("{} not found in the bitcoin node", wallet_name).into());
            } else {
                debug!("wallet {} already loaded", wallet_name);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SavePsbtOptions {
    pub psbt: StringEncoding,
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
pub mod tests {
    use crate::offline::random::RandomOptions;
    use crate::{psbt_from_base64, Context, DescriptorPublicKey, MasterSecret, OfflineContext};
    use bitcoin::Network;
    use std::ops::Deref;
    use tempfile::TempDir;

    #[derive(Debug)]
    pub struct TestContext {
        pub context: OfflineContext,
        #[allow(unused)]
        datadir: TempDir, // must be here so directory isn't removed before dropping the object
    }

    impl TestContext {
        pub fn with_network(network: Network) -> Self {
            let datadir = TempDir::new().unwrap();
            let firma_datadir = format!("{}/", datadir.path().display());
            TestContext {
                context: OfflineContext {
                    context: Context {
                        network,
                        datadir: firma_datadir,
                        encryption_key: None,
                    },
                },
                datadir,
            }
        }
    }

    impl Default for TestContext {
        fn default() -> Self {
            Self::with_network(Network::Testnet)
        }
    }

    impl Deref for TestContext {
        type Target = OfflineContext;

        fn deref(&self) -> &Self::Target {
            &self.context
        }
    }

    #[test]
    fn test_write_keys() {
        let context = TestContext::default();
        let key_name = "a";
        let key = context
            .context
            .create_key(&RandomOptions {
                key_name: key_name.to_string(),
            })
            .unwrap();
        assert!(
            context.write_keys(&key).is_err(),
            "can overwrite key material"
        );
        let key_read: MasterSecret = context.read(&key_name).unwrap();
        assert_eq!(key, key_read);
        let _: DescriptorPublicKey = context.read(&key_name).unwrap();
    }

    #[test]
    fn test_save_psbt() {
        let context = TestContext::default();
        let without_name= "cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAAAQChAgAAAAEbuYvreUkM84tDJuxdjxZmErxAyO/PkP+ozooG1kBiZAAAAAAjIgAg/KddPamHVwK3NnYT58PR3q+a5k9zwFC8zJXE6Nwr5zX9////AkyLBgAAAAAAF6kUZ3Eos+P2CT0g41zAxb+TPZLthgiHpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qn6FHQABASukzgAAAAAAACIAIPWRVyIca9Av7u6ho2/suW3Z5lT52N7R+fN7dkvB8j6qAQVHUiECkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4hA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gUq4iBgKSs5xbbfPnypRjnI4Cs8JhIMvDtnre2rZTX3QOW2/NLhyi6+BOMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDfXRZ174HHQOH5i1m5cCiW7s2s7oEt3c3Oj65Q6tn3eAcH15D2DAAAIABAACAAAAAgAIAAIAAAAAAAAAAAAAAAQFHUiEC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAhA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/Uq4iAgLjgp6MBzab6rhhE/ElAh6pu5UspqkrJbtlOAdlXWfYQByi6+BOMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDqOISXqlaOQ55ST8/9mFJ96Hci1iByNRgThG83aJCUX8cH15D2DAAAIABAACAAAAAgAIAAIAAAAAAAQAAAAA=";
        let (_, mut psbt) = psbt_from_base64(without_name).unwrap();
        let name = context.save_psbt(&mut psbt).unwrap();
        assert_eq!(name, "psbt-0");

        let context = TestContext::default(); // new context, so I don't get PSBTNotChangedAfterMerge
        let with_name = "cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAM/AVmaXJtYQBuYW1lCHRvLWNhcm9sAAEAoQIAAAABG7mL63lJDPOLQybsXY8WZhK8QMjvz5D/qM6KBtZAYmQAAAAAIyIAIPynXT2ph1cCtzZ2E+fD0d6vmuZPc8BQvMyVxOjcK+c1/f///wJMiwYAAAAAABepFGdxKLPj9gk9IONcwMW/kz2S7YYIh6TOAAAAAAAAIgAg9ZFXIhxr0C/u7qGjb+y5bdnmVPnY3tH583t2S8HyPqp+hR0AAQErpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qgEFR1IhApKznFtt8+fKlGOcjgKzwmEgy8O2et7atlNfdA5bb80uIQN9dFnXvgcdA4fmLWblwKJbuzazugS3dzc6PrlDq2fd4FKuIgYCkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4couvgTjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gHB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAAAEBR1IhAuOCnowHNpvquGET8SUCHqm7lSymqSslu2U4B2VdZ9hAIQOo4hJeqVo5DnlJPz/2YUn3odyLWIHI1GBOEbzdokJRf1KuIgIC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAcouvgTjAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/HB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAEAAAAA";
        let (_, mut psbt) = psbt_from_base64(with_name).unwrap();
        let name = context.save_psbt(&mut psbt).unwrap();
        assert_eq!(name, "to-carol");

        let context = TestContext::default(); // new context, so I don't get PSBTNotChangedAfterMerge
        let with_name = "cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAM/AVmaXJtYQBuYW1lCHRvLWNhcm9sAAEAoQIAAAABG7mL63lJDPOLQybsXY8WZhK8QMjvz5D/qM6KBtZAYmQAAAAAIyIAIPynXT2ph1cCtzZ2E+fD0d6vmuZPc8BQvMyVxOjcK+c1/f///wJMiwYAAAAAABepFGdxKLPj9gk9IONcwMW/kz2S7YYIh6TOAAAAAAAAIgAg9ZFXIhxr0C/u7qGjb+y5bdnmVPnY3tH583t2S8HyPqp+hR0AAQErpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qgEFR1IhApKznFtt8+fKlGOcjgKzwmEgy8O2et7atlNfdA5bb80uIQN9dFnXvgcdA4fmLWblwKJbuzazugS3dzc6PrlDq2fd4FKuIgYCkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4couvgTjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gHB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAAAEBR1IhAuOCnowHNpvquGET8SUCHqm7lSymqSslu2U4B2VdZ9hAIQOo4hJeqVo5DnlJPz/2YUn3odyLWIHI1GBOEbzdokJRf1KuIgIC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAcouvgTjAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/HB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAEAAAAA";
        let (_, mut psbt) = psbt_from_base64(with_name).unwrap();
        let name = context.save_psbt(&mut psbt).unwrap();
        assert_eq!(name, "to-carol");
    }
}
