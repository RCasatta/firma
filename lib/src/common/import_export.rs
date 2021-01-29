use crate::common::json::identifier::Identifier;
use crate::offline::decrypt::decrypt;
use crate::online::PathOptions;
use crate::*;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    /// The kind of the object to export
    #[structopt(long)]
    pub kind: Kind,

    /// The name of the object to export
    #[structopt(long)]
    pub name: String,
}

impl Context {
    pub fn import(&self, opt: &PathOptions) -> Result<Value> {
        let value: Value = decrypt(&opt.path, &self.encryption_key)?;
        self.import_json(value)
    }

    pub fn import_json(&self, value: Value) -> Result<Value> {
        let id_value = value.get("id").ok_or(Error::MissingIdentifier)?;
        let id: Identifier = from_value(id_value.clone())?;
        let c = value.clone();
        match id.kind {
            Kind::Wallet => self.write(&from_value::<WalletJson>(c)?)?,
            Kind::WalletIndexes => self.write(&from_value::<IndexesJson>(c)?)?,
            Kind::WalletSignature => self.write(&from_value::<WalletSignatureJson>(c)?)?,
            Kind::MasterSecret => self.write(&from_value::<MasterSecretJson>(c)?)?,
            Kind::DescriptorPublicKey => self.write(&from_value::<PublicMasterKey>(c)?)?,
            Kind::PSBT => self.write(&from_value::<PsbtJson>(c)?)?,
        }
        Ok(value)
    }

    pub fn export(&self, opt: &ExportOptions) -> Result<Value> {
        debug!("export {:?}", opt);
        let id = Identifier::new(self.network, opt.kind, &opt.name);
        id.read(&self.datadir, &self.encryption_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::context::tests::TestContext;
    use crate::online::PathOptions;
    use crate::{Identifier, IndexesJson, Kind};

    #[test]
    fn test_import() {
        let context = TestContext::new();
        let i = IndexesJson {
            id: Identifier::new_test(Kind::WalletIndexes),
            main: 0,
        };
        context.write(&i).unwrap();
        let path = i.id.as_path_buf(&context.datadir, false).unwrap();
        let second_context = TestContext::new();
        assert!(second_context.read::<IndexesJson>(&i.id.name).is_err());
        second_context.import(&PathOptions { path }).unwrap();
        let read = second_context.read::<IndexesJson>(&i.id.name).unwrap();
        assert_eq!(i, read);
    }
}
