use crate::common::json::identifier::Identifier;
use crate::online::PathOptions;
use crate::*;
use serde_json::{from_value, Value};

impl Context {
    pub fn import(&self, opt: &PathOptions) -> Result<Value> {
        let bytes = std::fs::read(&opt.path)?;
        let value = serde_json::from_slice(&bytes)?;
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
}

#[cfg(test)]
mod tests {
    use crate::{IndexesJson, Identifier, Kind};
    use crate::context::tests::TestContext;
    use crate::online::PathOptions;

    #[test]
    fn test_import() {
        let context = TestContext::new();
        let i = IndexesJson { id: Identifier::new_test(Kind::WalletIndexes), main: 0 };
        context.write(&i).unwrap();
        let path = i.id.as_path_buf(&context.firma_datadir, false).unwrap();
        let second_context = TestContext::new();
        assert!(second_context.read::<IndexesJson>(&i.id.name).is_err());
        second_context.import(&PathOptions { path }).unwrap();
        let read = second_context.read::<IndexesJson>(&i.id.name).unwrap();
        assert_eq!(i, read);
    }
}