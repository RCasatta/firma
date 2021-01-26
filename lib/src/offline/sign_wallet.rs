use crate::common::json::identifier::{Identifier, Kind};
use crate::common::list::ListOptions;
use crate::offline::descriptor::extract_xpubs;
use crate::*;
use bitcoin::secp256k1::recovery::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1, SignOnly, VerifyOnly};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::util::misc::signed_msg_hash;
use bitcoin::{Address, PrivateKey, PublicKey};
use log::debug;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Serialize, Deserialize, StructOpt, Debug)]
pub struct SignWalletOptions {
    /// Wallet name to be signed
    #[structopt(long)]
    pub wallet_name: String,
}

#[derive(Serialize, Deserialize, StructOpt, Debug)]
pub struct VerifyWalletOptions {
    /// Wallet name to be verified
    #[structopt(long)]
    pub wallet_name: String,
}

impl Context {
    pub fn verify_wallet(&self, opt: &VerifyWalletOptions) -> Result<VerifyWalletResult> {
        let wallet: WalletJson = self.read(&opt.wallet_name)?;
        let signature: WalletSignatureJson = self.read(&opt.wallet_name)?;
        let secp = Secp256k1::verification_only();

        verify_wallet_internal(&wallet, &signature, &secp)
    }
}

impl Context {
    pub fn sign_wallet(&self, opt: &SignWalletOptions) -> Result<WalletSignatureJson> {
        let secp = Secp256k1::signing_only();
        let wallet: WalletJson = self.read(&opt.wallet_name)?;
        let message = &wallet.descriptor;
        let xpubs: Vec<ExtendedPubKey> = extract_xpubs(&wallet.descriptor)?;

        // search a key that is in the wallet descriptor
        let kind = Kind::MasterSecret;
        let list_opt = ListOptions {
            kind,
            verify_wallets_signatures: false,
        };
        debug!("list_opt {:?}", list_opt);
        let available_keys = self.list(&list_opt)?;
        let master_private_key = find_key(&available_keys, &xpubs)?; // TODO should be added a derivation?
        let master_public_key = ExtendedPubKey::from_private(&secp, &master_private_key);

        let signature = sign_message_with_key(&master_private_key.private_key, message, &secp)?;
        let address = Address::p2pkh(&master_public_key.public_key, master_public_key.network);

        xpubs
            .iter()
            .try_for_each(|xpub| check_compatibility(self.network, xpub.network))?;

        let wallet_signature = WalletSignatureJson {
            xpub: master_public_key,
            address,
            signature,
            id: Identifier::new(self.network, Kind::WalletSignature, &wallet.id.name),
        };

        self.write(&wallet_signature)?;
        Ok(wallet_signature)
    }
}

fn find_key<'a>(
    available_keys: &'a ListOutput,
    xpubs: &[ExtendedPubKey],
) -> Result<&'a ExtendedPrivKey> {
    for key in available_keys.keys.iter() {
        if check_xpub_in_descriptor(&key.key.xpub, &xpubs).is_ok() {
            return Ok(&key.key.xprv);
        }
    }
    Err("There is No private key participating in the wallet available".into())
}

fn check_xpub_in_descriptor(
    master_public_key: &ExtendedPubKey,
    xpubs: &[ExtendedPubKey],
) -> Result<()> {
    let is_wallet_key = xpubs.iter().any(|e| e == master_public_key);
    if !is_wallet_key {
        Err("Provided key is not part of this multisig wallet".into())
    } else {
        Ok(())
    }
}

pub fn verify_wallet_internal(
    wallet: &WalletJson,
    signature: &WalletSignatureJson,
    secp: &Secp256k1<VerifyOnly>,
) -> Result<VerifyWalletResult> {
    let xpubs = extract_xpubs(&wallet.descriptor)?;
    let message = &wallet.descriptor;
    let master_address = Address::p2pkh(&signature.xpub.public_key, signature.xpub.network);

    check_xpub_in_descriptor(&signature.xpub, &xpubs)?;
    debug!("xpub is in wallet");
    if master_address != signature.address {
        return Err("Address in signature does not match master xpub address".into());
    }
    debug!("address matches");
    let verified =
        verify_message_with_address(&signature.address, &signature.signature, message, secp)?;
    debug!("verified {}", verified);
    let result = VerifyWalletResult {
        descriptor: wallet.descriptor.to_string(),
        signature: signature.clone(),
        verified,
    };
    Ok(result)
}

fn sign_message_with_key(
    private_key: &PrivateKey,
    message: &str,
    secp: &Secp256k1<SignOnly>,
) -> Result<String> {
    let hash = signed_msg_hash(&message);
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
pub fn sign_message(private_key: &str, message: &str) -> Result<String> {
    let secp = Secp256k1::signing_only();
    let private_key = PrivateKey::from_wif(&private_key)?;
    sign_message_with_key(&private_key, message, &secp)
}

fn verify_message_with_address(
    address: &Address,
    signature: &str,
    message: &str,
    secp: &Secp256k1<VerifyOnly>,
) -> Result<bool> {
    let sig = base64::decode(&signature)?;
    if sig.len() != 65 {
        return Err(Error::InvalidMessageSignature);
    }
    let recid = RecoveryId::from_i32(i32::from((sig[0] - 27) & 3))
        .map_err(|_| Error::InvalidMessageSignature)?;
    let recsig = RecoverableSignature::from_compact(&sig[1..], recid)
        .map_err(|_| Error::InvalidMessageSignature)?;
    let hash = signed_msg_hash(&message);
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
fn verify_message(address: &str, signature: &str, message: &str) -> Result<bool> {
    let secp = Secp256k1::verification_only();
    let address = Address::from_str(&address)?;
    verify_message_with_address(&address, signature, message, &secp)
}

// json contains signature, the address and descriptor of the address!
#[cfg(test)]
mod tests {
    use crate::offline::sign_wallet::{sign_message, verify_message};

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
        assert_eq!(SIGNATURE, sign_message(PRIV_WIF, MESSAGE).unwrap());
    }

    #[test]
    fn test_verify_message() {
        assert!(verify_message(ADDRESS, SIGNATURE, MESSAGE).unwrap());
    }
}
