use crate::{Error, Result, StringEncoding};
use aes_gcm_siv::aead::generic_array::GenericArray;
use aes_gcm_siv::aead::{Aead, NewAead};
use aes_gcm_siv::Aes256GcmSiv;
use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct DecryptOptions {
    /// File containing the encrypted data
    #[structopt(short, long, parse(from_os_str))]
    pub path: PathBuf,

    /// in CLI it is populated from standard input
    /// It is an Option so that structopt could skip, however it must be Some
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

pub fn decrypt<T>(opt: &DecryptOptions) -> Result<T>
where
    T: Serialize + DeserializeOwned,
{
    let file_content = std::fs::read(&opt.path)?;
    let maybe_encrypted: MaybeEncrypted<T> = serde_json::from_slice(&file_content)?;
    match (maybe_encrypted, opt.encryption_key.clone()) {
        (MaybeEncrypted::Plain(value), None) => Ok(value),
        (maybe_encrypted @ MaybeEncrypted::Encrypted(_), Some(encryption_key)) => {
            match maybe_encrypted.decrypt(&encryption_key.get_exactly_32()?) {
                Ok(MaybeEncrypted::Plain(value)) => Ok(value),
                _ => Err(Error::MaybeEncryptedWrongState),
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
    T: Serialize + DeserializeOwned,
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

#[derive(Debug, Serialize)]
struct S {
    a: u32,
}

#[cfg(test)]
mod tests {
    use crate::common::mnemonic::Mnemonic;
    use crate::offline::decrypt::MaybeEncrypted;
    use crate::PrivateMasterKey;
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
        let maybe_encrypt = maybe_plain.encrypt(&cipher_key).unwrap();
        assert!(!serde_json::to_string(&maybe_encrypt)
            .unwrap()
            .contains(tpub_str));
        let maybe_plain_again = maybe_encrypt.decrypt(&cipher_key).unwrap();
        assert_eq!(maybe_plain, maybe_plain_again);

        let key_json = PrivateMasterKey::new(
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
