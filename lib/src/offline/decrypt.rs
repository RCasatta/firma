use crate::{Error, Result, StringEncoding};
use aes_gcm_siv::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm_siv::Aes256GcmSiv;
use log::warn;
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;
use structopt::StructOpt;

/*let mut maybe_encrypted = MaybeEncrypted::plain(private_key.clone());
if let Some(encryption_key) = encryption_key {
    maybe_encrypted = maybe_encrypted.encrypt(&encryption_key.get_exactly_32()?)?;
}
save(serde_json::to_string_pretty(&maybe_encrypted)?, output)*/

/*
let decrypted = match encryption_key {
    encryption_key @ Some(_) => {
        MaybeEncrypted::Plain(decrypt(&DecryptOptions::new(path, encryption_key))?)
    }
    None => serde_json::from_slice(&std::fs::read(path)?)?,
};
match decrypted {
    MaybeEncrypted::Plain(value) => Ok(value),
    MaybeEncrypted::Encrypted(_) => Err(Error::MaybeEncryptedWrongState),
}

 */

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct DecryptOptions {
    /// File containing the encrypted data
    #[structopt(short, long, parse(from_os_str))]
    pub path: PathBuf,

    /// in CLI it is populated from standard input
    /// It is an Option so that structopt could skip, however it must be to decrypt
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

pub fn decrypt<T>(opt: &DecryptOptions) -> Result<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    let file_content = std::fs::read(&opt.path)?;
    let maybe_encrypted: MaybeEncrypted<T> = serde_json::from_slice(&file_content)?;
    match (maybe_encrypted, opt.encryption_key.clone()) {
        (MaybeEncrypted::Plain(value), None) => Ok(value),
        (maybe_encrypted @ MaybeEncrypted::Encrypted(_), Some(encryption_key)) => {
            match maybe_encrypted.decrypt(&encryption_key.get_exactly_32()?) {
                Ok(MaybeEncrypted::Plain(value)) => Ok(value),
                Ok(_) => Err(Error::MaybeEncryptedWrongState),
                Err(e) => {
                    warn!("Other error {:?}", e);
                    Err(e)
                }
            }
        }
        _ => Err(Error::MaybeEncryptedWrongState),
    }
}

impl DecryptOptions {
    pub fn new(path: &PathBuf, encryption_key: Option<&StringEncoding>) -> Self {
        DecryptOptions {
            path: path.clone(),
            encryption_key: encryption_key.cloned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "t", content = "c", rename_all = "lowercase")]
pub enum MaybeEncrypted<T> {
    Plain(T),
    Encrypted(StringEncoding),
}

impl<T> MaybeEncrypted<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    pub fn plain(element: T) -> Self {
        MaybeEncrypted::Plain(element)
    }

    pub fn encrypt(&self, encryption_key: &[u8; 32]) -> crate::Result<Self> {
        match self {
            MaybeEncrypted::Plain(plaintext) => {
                let cipher = get_cipher(encryption_key);
                let mut nonce_bytes = [0u8; 12]; // Suggested 96 bits
                thread_rng().fill(&mut nonce_bytes);
                let nonce = GenericArray::from_slice(&nonce_bytes);
                let plaintext = serde_json::to_vec(plaintext)?;
                let ciphertext = cipher.encrypt(&nonce, &plaintext[..])?;
                let mut result = nonce_bytes.to_vec();
                result.extend(ciphertext);
                Ok(MaybeEncrypted::Encrypted(StringEncoding::new_base64(
                    &result,
                )))
            }
            MaybeEncrypted::Encrypted(_) => Err(Error::MaybeEncryptedWrongState),
        }
    }

    pub fn decrypt(&self, encryption_key: &[u8; 32]) -> crate::Result<Self> {
        match self {
            MaybeEncrypted::Plain(_) => Err(Error::MaybeEncryptedWrongState),
            MaybeEncrypted::Encrypted(ciphertext) => {
                let cipher = get_cipher(encryption_key);
                let ciphertext = ciphertext.as_bytes()?;
                let nonce_bytes = &ciphertext[0..12];
                let nonce = GenericArray::from_slice(&nonce_bytes);
                let plaintext = cipher.decrypt(&nonce, &ciphertext[12..])?;
                let result = serde_json::from_slice(&plaintext)?;
                Ok(MaybeEncrypted::Plain(result))
            }
        }
    }
}

fn get_cipher(encryption_key: &[u8; 32]) -> Aes256GcmSiv {
    let encryption_key = GenericArray::from_slice(&encryption_key[..]);
    Aes256GcmSiv::new(&encryption_key)
}

#[cfg(test)]
mod tests {
    use crate::common::mnemonic::Mnemonic;
    use crate::offline::decrypt::MaybeEncrypted;
    use crate::MasterSecretJson;
    use bitcoin::util::bip32::ExtendedPubKey;
    use bitcoin::Network;
    use rand::{thread_rng, Rng};
    use std::str::FromStr;

    #[test]
    fn test_maybe_encrypted_rt() {
        let mut cipher_key = [0u8; 32];
        thread_rng().fill(&mut cipher_key);
        let tpub_str = "tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU";
        let tpub = ExtendedPubKey::from_str(tpub_str).unwrap();
        let maybe_plain = MaybeEncrypted::plain(tpub);
        assert!(
            maybe_plain.decrypt(&cipher_key).is_err(),
            "cannot decrypt plaintext"
        );
        let maybe_encrypt = maybe_plain.encrypt(&cipher_key).unwrap();
        assert!(!serde_json::to_string(&maybe_encrypt)
            .unwrap()
            .contains(tpub_str));
        assert!(
            maybe_encrypt.encrypt(&cipher_key).is_err(),
            "cannot encrypt ciphertext"
        );
        let maybe_plain_again = maybe_encrypt.decrypt(&cipher_key).unwrap();
        assert_eq!(maybe_plain, maybe_plain_again);

        let key_json = MasterSecretJson::new(
            Network::Testnet,
            &Mnemonic::from_str(
                "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            )
            .unwrap(),
            "ciao",
        )
        .unwrap();
        let maybe_plain = MaybeEncrypted::plain(key_json);
        let maybe_encrypt = maybe_plain.encrypt(&cipher_key).unwrap();
        let maybe_plain_again = maybe_encrypt.decrypt(&cipher_key).unwrap();
        assert_eq!(maybe_plain, maybe_plain_again);
    }
}
