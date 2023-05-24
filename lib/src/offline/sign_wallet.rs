use crate::common::list::ListOptions;
use crate::online::WalletNameOptions;
use crate::*;
use bitcoin::secp256k1::recovery::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1, Signing, Verification};
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::util::misc::signed_msg_hash;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use log::debug;
use std::str::FromStr;

pub const WALLET_SIGN_DERIVATION: u32 = u32::MAX >> 1;

impl OfflineContext {
    pub fn verify_wallet(&self, opt: &WalletNameOptions) -> Result<VerifyWalletResult> {
        let secp = Secp256k1::verification_only();
        let wallet: Wallet = self.read(&opt.wallet_name)?;
        let signature: WalletSignature = self.read(&opt.wallet_name)?;

        verify_wallet_internal(&secp, &wallet, &signature, self.network)
    }

    pub fn sign_wallet(&self, opt: &WalletNameOptions) -> Result<WalletSignature> {
        let secp = Secp256k1::signing_only();
        let wallet: Wallet = self.read(&opt.wallet_name)?;
        let message = &wallet.descriptor;
        debug!("sign_wallet descriptor: {}", message);
        let desc_pub_keys: Vec<PublicKey> = wallet.extract_wallet_sign_keys()?;
        debug!(
            "sign_wallet desc_pub_keys: {}",
            desc_pub_keys
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );

        // search a key that is in the wallet descriptor
        let kind = Kind::MasterSecret;
        let list_opt = ListOptions { kind };
        debug!("sign_wallet list_opt {:?}", list_opt);
        let available_keys = self.list(&list_opt)?;
        let master_private_key = find_key(&secp, &available_keys, &desc_pub_keys)?;
        debug!("sign_wallet using {}", master_private_key.id.name);
        let key = master_private_key.as_wallet_sign_prv_key(&secp)?;
        let pub_key = master_private_key.as_wallet_sign_pub_key(&secp)?;
        debug!("sign_wallet using {}", pub_key);
        let match_pub = ExtendedPubKey::from_private(&secp, &key);
        assert_eq!(pub_key, match_pub.public_key);

        let signature = sign_message_with_key(&secp, &key.private_key, message)?;

        /*desc_pub_keys
           .iter()
           .try_for_each(|xpub| check_compatibility(self.network, xpub.network))?;
        */

        let wallet_signature = WalletSignature {
            signature,
            id: Identifier::new(self.network, Kind::WalletSignature, &wallet.id.name),
        };

        self.write(&wallet_signature)?;
        Ok(wallet_signature)
    }
}

fn find_key<'a, T: Signing>(
    secp: &Secp256k1<T>,
    available_keys: &'a ListOutput,
    desc_pub_keys: &[PublicKey],
) -> Result<&'a MasterSecret> {
    for key in available_keys.master_secrets.iter() {
        let k = key.as_wallet_sign_pub_key(secp)?;
        debug!("find_key key:{} -> sign_pub_key:{}", key.id.name, k);
        if desc_pub_keys.contains(&k) {
            debug!("find_key found pubkey {} of key {}", k, key.id.name);
            return Ok(key);
        }
    }
    Err("There is no private key participating in the wallet available".into())
}

pub fn verify_wallet_internal<T: Verification>(
    secp: &Secp256k1<T>,
    wallet: &Wallet,
    signature: &WalletSignature,
    network: Network,
) -> Result<VerifyWalletResult> {
    let desc_pub_keys = wallet.extract_desc_pub_keys()?;
    let message = &wallet.descriptor;

    for desc_pub_key in desc_pub_keys {
        debug!("verify_wallet_internal desc_pub_key:{}", desc_pub_key);
        let pubkey = desc_pub_key
            .derive(WALLET_SIGN_DERIVATION)
            .derive_public_key(secp)
            .unwrap(); //TODO
        debug!("verify_wallet_internal pubkey:{}", pubkey);
        let master_address = Address::p2pkh(&pubkey, network);
        let verified =
            verify_message_with_address(secp, &master_address, &signature.signature, message)?;
        debug!(
            "with master_address {} verified {}",
            master_address, verified
        );
        if verified {
            let result = VerifyWalletResult {
                descriptor: wallet.descriptor.to_string(),
                signature: signature.clone(),
                verified,
            };
            return Ok(result);
        }
    }

    Err(Error::WalletSignatureNotVerified)
}

fn sign_message_with_key<T: Signing>(
    secp: &Secp256k1<T>,
    private_key: &PrivateKey,
    message: &str,
) -> Result<String> {
    let hash = signed_msg_hash(message);
    debug!("Signed message hash:{}", hash);
    let message = Message::from_slice(&hash[..])?; // Can never panic because it's the right size.

    let (id, sig) = secp
        .sign_recoverable(&message, &private_key.key)
        .serialize_compact();
    let mut rec_sig = [0u8; 65];
    rec_sig[1..].copy_from_slice(&sig);
    rec_sig[0] = if private_key.compressed {
        27 + id.to_i32() as u8 + 4
    } else {
        27 + id.to_i32() as u8
    };
    let sig = base64::encode(&rec_sig[..]);
    Ok(sig)
}

/// Sign a `message` with the given `private_key` in wif format
/// compatible with bitcoin core `signmessagewithprivkey`
pub fn sign_message<T: Signing>(
    secp: &Secp256k1<T>,
    private_key: &str,
    message: &str,
) -> Result<String> {
    let private_key = PrivateKey::from_wif(private_key)?;
    sign_message_with_key(secp, &private_key, message)
}

fn verify_message_with_address<T: Verification>(
    secp: &Secp256k1<T>,
    address: &Address,
    signature: &str,
    message: &str,
) -> Result<bool> {
    let sig = base64::decode(signature)?;
    if sig.len() != 65 {
        return Err(Error::InvalidMessageSignature);
    }
    let recid = RecoveryId::from_i32(i32::from((sig[0] - 27) & 3))
        .map_err(|_| Error::InvalidMessageSignature)?;
    let recsig = RecoverableSignature::from_compact(&sig[1..], recid)
        .map_err(|_| Error::InvalidMessageSignature)?;
    let hash = signed_msg_hash(message);
    let msg = Message::from_slice(&hash[..]).unwrap(); // Can never panic because it's the right size.

    let pubkey = PublicKey {
        key: secp
            .recover(&msg, &recsig)
            .map_err(|_| Error::InvalidMessageSignature)?,
        compressed: ((sig[0] - 27) & 4) != 0,
    };

    let restored = Address::p2pkh(&pubkey, address.network);

    Ok(address == &restored)
}

#[allow(dead_code)]
/// Verify the `signature` on a `message` has been made from the private key behind `address`
/// signature made must be recoverable
/// compatible with bitcoin core `verifymessage`
fn verify_message<T: Verification>(
    secp: &Secp256k1<T>,
    address: &str,
    signature: &str,
    message: &str,
) -> Result<bool> {
    let address = Address::from_str(address)?;
    verify_message_with_address(secp, &address, signature, message)
}

// json contains signature, the address and descriptor of the address!
#[cfg(test)]
mod tests {
    use crate::context::tests::TestContext;
    use crate::offline::random::RandomOptions;
    use crate::offline::sign_wallet::{sign_message, verify_message};
    use crate::online::WalletNameOptions;
    use crate::{Error, Wallet};
    use bitcoin::secp256k1::Secp256k1;
    use bitcoin::util::bip32::ExtendedPubKey;
    use bitcoin::Network;

    /*
    $ bitcoin-cli signmessagewithprivkey "KwQoPt6dL91fxRBWdt4nkCVrfo4ipeLcaD4ZCLntoTPhKGNgGqGm" ciao
    IPJtNiCerA3gbXxSIMzmrUyFeeL0BT/BM0nQU43XRl9QBuZkSnlotcNAp0cg6VqTRCjJkxwg0KTtJS96YcnjzRs=
    $ bitcoin-cli verifymessage 1AupUZ3sAdTjZSdG4D52eFoHdPtjwGZrTj "IPJtNiCerA3gbXxSIMzmrUyFeeL0BT/BM0nQU43XRl9QBuZkSnlotcNAp0cg6VqTRCjJkxwg0KTtJS96YcnjzRs=" ciao
    true
    */
    const PRIV_WIF: &str = "KwQoPt6dL91fxRBWdt4nkCVrfo4ipeLcaD4ZCLntoTPhKGNgGqGm";
    const MESSAGE: &str = "ciao";
    const SIGNATURE: &str =
        "IPJtNiCerA3gbXxSIMzmrUyFeeL0BT/BM0nQU43XRl9QBuZkSnlotcNAp0cg6VqTRCjJkxwg0KTtJS96YcnjzRs=";
    const ADDRESS: &str = "1AupUZ3sAdTjZSdG4D52eFoHdPtjwGZrTj";

    #[test]
    fn test_sign_message() {
        let secp = Secp256k1::signing_only();
        assert_eq!(SIGNATURE, sign_message(&secp, PRIV_WIF, MESSAGE).unwrap());
    }

    #[test]
    fn test_verify_message() {
        let secp = Secp256k1::verification_only();
        assert!(verify_message(&secp, ADDRESS, SIGNATURE, MESSAGE).unwrap());
    }

    #[test]
    fn test_sign_key() {
        let secp = Secp256k1::signing_only();
        for network in [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ]
        .iter()
        {
            let context = TestContext::with_network(*network);
            let master_private_key = context.create_key(&RandomOptions::new_random()).unwrap();
            let key = master_private_key.as_wallet_sign_prv_key(&secp).unwrap();
            let pub_key = master_private_key.as_wallet_sign_pub_key(&secp).unwrap();
            let match_pub = ExtendedPubKey::from_private(&secp, &key);
            assert_eq!(pub_key, match_pub.public_key);
        }
    }

    #[test]
    fn test_sign_verify() {
        let context = TestContext::default();
        let key = context.create_key(&RandomOptions::new_random()).unwrap();
        let wallet = Wallet::new_random(1, &vec![key]);
        let wallet_name_opt: WalletNameOptions = wallet.id.name.as_str().into();

        // manually importing the wallet, because context.create_wallet needs the node, not available in unit tests
        context
            .import_json(serde_json::to_value(wallet).unwrap())
            .unwrap();

        let err = context.verify_wallet(&wallet_name_opt);
        assert_matches!(err, Err(Error::FileNotFoundOrCorrupt(..)));
        let mut signature = context.sign_wallet(&wallet_name_opt).unwrap();
        let result = context.verify_wallet(&wallet_name_opt).unwrap();
        assert!(result.verified, "valid signature did not verify");

        let path = signature.id.as_path_buf(&context.datadir, false).unwrap();
        std::fs::remove_file(path).unwrap();
        let _ = context.verify_wallet(&wallet_name_opt).unwrap_err();

        let key_2 = context.create_key(&RandomOptions::new_random()).unwrap();
        let wallet_2 = Wallet::new_random(1, &vec![key_2]);
        let wallet_2_name_opt: WalletNameOptions = wallet_2.id.name.as_str().into();
        context
            .import_json(serde_json::to_value(wallet_2).unwrap())
            .unwrap();
        let signature_2 = context.sign_wallet(&wallet_2_name_opt).unwrap();
        let result_2 = context.verify_wallet(&wallet_2_name_opt).unwrap();
        assert!(result_2.verified, "valid signature did not verify");

        signature.signature = signature_2.signature;
        context
            .import_json(serde_json::to_value(signature).unwrap())
            .unwrap();
        let err = context.verify_wallet(&wallet_name_opt);
        assert_matches!(err, Err(Error::WalletSignatureNotVerified));
    }
}
