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

impl Context {
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
    use crate::offline::random::RandomOptions;

    #[test]
    fn test_random() {
        let key_name = "random".to_string();
        let rand_opts_1 = RandomOptions { key_name };
        let context = TestContext::new();
        let key_1 = context.create_key(&rand_opts_1).unwrap();
        let result = context.create_key(&rand_opts_1);
        assert!(result.is_err(), "Overwrite a key");
        assert!(result.unwrap_err().to_string().contains("Cannot overwrite"));

        let key_name = "random_2".to_string();
        let rand_opts_2 = RandomOptions { key_name };
        let key_2 = context.create_key(&rand_opts_2).unwrap();

        assert_ne!(key_1, key_2);
    }
}
