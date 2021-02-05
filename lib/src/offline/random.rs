use crate::*;
use common::mnemonic::Mnemonic;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Generate a bitcoin master key in bip32 randomly
#[derive(StructOpt, Debug, Serialize, Deserialize, Clone)]
#[structopt(name = "random")]
pub struct RandomOptions {
    /// Name of the key
    #[structopt(short, long)]
    pub key_name: String,
}

impl OfflineContext {
    pub fn create_key(&self, opt: &RandomOptions) -> Result<MasterSecretJson> {
        let sec = rand::thread_rng().gen::<[u8; 32]>();
        let mnemonic = Mnemonic::new(&sec)?;
        let master_key = MasterSecretJson::new(self.network, &mnemonic, &opt.key_name)?;
        self.write_keys(&master_key)?;

        Ok(master_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::context::tests::TestContext;
    use crate::common::tests::rnd_string;
    use crate::offline::random::RandomOptions;

    impl RandomOptions {
        pub fn new_random() -> Self {
            let key_name = rnd_string();
            RandomOptions { key_name }
        }
    }

    #[test]
    fn test_random() {
        let rand_opts_1 = RandomOptions::new_random();
        let context = TestContext::default();
        let key_1 = context.create_key(&rand_opts_1).unwrap();
        let result = context.create_key(&rand_opts_1);
        assert!(result.is_err(), "Overwrite a key");
        assert!(result.unwrap_err().to_string().contains("Cannot overwrite"));

        let key_2 = context.create_key(&RandomOptions::new_random()).unwrap();

        assert_ne!(key_1, key_2);
    }
}
