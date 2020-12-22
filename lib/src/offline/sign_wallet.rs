use crate::common::list::ListOptions;
use crate::offline::descriptor::extract_xpubs;
use crate::*;
use bitcoin::secp256k1::recovery::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1, SignOnly, VerifyOnly};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::util::misc::signed_msg_hash;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Serialize, Deserialize, StructOpt, Debug)]
pub struct SignWalletOptions {
    /// Wallet name to be signed
    #[structopt(long)]
    pub wallet_name: String,

    /// in CLI it is populated from standard input
    /// It is an Option so that structopt could skip,
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

pub fn sign_wallet(
    datadir: &str,
    network: Network,
    opt: &SignWalletOptions,
) -> Result<WalletSignature> {
    let secp = Secp256k1::signing_only();

    let wallet_file = PathBuilder::new(
        datadir,
        network,
        Kind::Wallet,
        Some(opt.wallet_name.to_string()),
    )
    .file("descriptor.json")?;
    debug!("wallet_file {:?}", wallet_file);
    let wallet = read_wallet(&wallet_file)?; // read the json
    let wallet_bytes = fs::read(wallet_file)?; // reading exact bytes to sign, (reserializing wallet would be wrong cause it's not guaranteed field are in the same order)
    let message = std::str::from_utf8(&wallet_bytes)?;
    let xpubs: Vec<ExtendedPubKey> = extract_xpubs(&wallet.descriptor)?;
    let encryption_keys = match opt.encryption_key.as_ref() {
        Some(key) => vec![key.clone()],
        None => vec![],
    };

    // search a key that is in the wallet descriptor
    let kind = Kind::Key;
    let list_opt = ListOptions {
        kind,
        verify_wallets_signatures: false,
        encryption_keys,
    };
    let available_keys = common::list::list(datadir, network, &list_opt)?;
    let master_private_key = find_key(&available_keys, &xpubs)?; // TODO should be added a derivation?
    let master_public_key = ExtendedPubKey::from_private(&secp, &master_private_key);

    let signature = sign_message_with_key(&master_private_key.private_key, message, &secp)?;
    let address = Address::p2pkh(&master_public_key.public_key, master_public_key.network);

    xpubs
        .iter()
        .map(|xpub| check_compatibility(network, xpub.network))
        .collect::<Result<()>>()?;
    let context = Context {
        firma_datadir: datadir.to_string(),
        network,
        wallet_name: wallet.name,
    };

    let wallet_signature = WalletSignature {
        xpub: master_public_key,
        address,
        signature,
    };
    context.save_signature(&wallet_signature)?;
    Ok(wallet_signature)
}

fn find_key<'a>(
    available_keys: &'a ListOutput,
    xpubs: &[ExtendedPubKey],
) -> Result<&'a ExtendedPrivKey> {
    for key in available_keys.keys.iter() {
        if let Ok(_) = check_xpub_in_descriptor(&key.key.xpub, &xpubs) {
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

pub fn verify_wallet(
    wallet_path: &PathBuf,
    signature_path: &PathBuf,
    secp: &Secp256k1<VerifyOnly>,
) -> Result<bool> {
    let wallet = read_wallet(&wallet_path)?;
    let wallet_bytes = fs::read(&wallet_path)?;
    let signature = read_signature(&signature_path)?;
    let xpubs = extract_xpubs(&wallet.descriptor)?;
    let message = std::str::from_utf8(&wallet_bytes)?;
    let master_address = Address::p2pkh(&signature.xpub.public_key, signature.xpub.network);

    check_xpub_in_descriptor(&signature.xpub, &xpubs)?;
    if master_address != signature.address {
        return Err("Address in signature does not match master xpub address".into());
    }
    verify_message_with_address(&signature.address, &signature.signature, message, secp)
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
