use crate::offline::descriptor::extract_xpubs;
use crate::offline::sign::read_key;
use crate::*;
use bitcoin::secp256k1::recovery::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1, SignOnly};
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::util::misc::signed_msg_hash;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ImportWalletOptions {
    /// Path of the wallet json to import, this or `wallet_encoded` must be set
    #[structopt(long = "wallet-path")]
    pub wallet_path: Option<PathBuf>,

    /// Wallet json encoded, this or `wallet_path` must be set
    #[structopt(skip)]
    pub wallet_encoded: Option<StringEncoding>,

    /// Private key used to sign the wallet to prevent tampering,
    /// the key must be part of the multisignature scheme
    #[structopt(long)]
    pub key_path: PathBuf,

    /// in CLI it is populated from standard input
    /// It is an Option so that structopt could skip,
    #[structopt(skip)]
    pub encryption_key: Option<StringEncoding>,
}

pub fn import_wallet(
    datadir: &str,
    network: Network,
    opt: &ImportWalletOptions,
) -> Result<WalletSignature> {
    let wallet = match (opt.wallet_path.as_ref(), opt.wallet_encoded.as_ref()) {
        (Some(a), None) => read_wallet(a)?,
        (None, Some(b)) => serde_json::from_slice(&b.as_bytes()?)?,
        (Some(_), Some(_)) => return Err("cannot set both wallet_path and wallet_encoded".into()),
        (None, None) => return Err("both wallet_path and wallet_encoded are not set".into()),
    };
    //TODO at the moment wallet file is saved with prettify but it seemed wrong to use that format
    // for the signature, thus here I am using plain vec, requiring something like "| jq -c" to
    // verify from shell tools, see verify-signature.md
    let wallet_bytes = serde_json::to_vec(&wallet)?;

    let secp = Secp256k1::signing_only();
    let master_private_key_json = read_key(&opt.key_path, opt.encryption_key.as_ref())?;
    let master_private_key = master_private_key_json.xprv; // TODO should be added a derivation?
    let master_public_key = ExtendedPubKey::from_private(&secp, &master_private_key);

    let message = std::str::from_utf8(&wallet_bytes)?;
    let signature = sign_message_with_key(&master_private_key.private_key, message, &secp)?;
    let address = Address::p2pkh(&master_public_key.public_key, master_public_key.network);
    let wallet_signature = WalletSignature {
        xpub: master_public_key,
        address,
        signature,
    };

    //TODO check it's part of the multisig

    extract_xpubs(&wallet.descriptor)?
        .iter()
        .map(|xpub| check_compatibility(network, xpub.network))
        .collect::<Result<()>>()?;
    let context = Context {
        firma_datadir: datadir.to_string(),
        network,
        wallet_name: wallet.name.clone(),
    };
    context.save_signature(&wallet_signature)?;
    context.save_wallet(&wallet)?;

    let wallet_qr_path = context.path_for_wallet_qr()?;
    common::qr::save_qrs(wallet_bytes, wallet_qr_path, 14)?;
    Ok(wallet_signature)
}

fn sign_message_with_key(
    private_key: &PrivateKey,
    message: &str,
    secp: &Secp256k1<SignOnly>,
) -> Result<String> {
    let hash = signed_msg_hash(&message);
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

#[allow(dead_code)]
/// Sign a `message` with the given `private_key` in wif format
/// compatible with bitcoin core `signmessagewithprivkey`
pub fn sign_message(private_key: &str, message: &str) -> Result<String> {
    let secp = Secp256k1::signing_only();
    let private_key = PrivateKey::from_wif(&private_key)?;
    sign_message_with_key(&private_key, message, &secp)
}

#[allow(dead_code)]
/// Verify the `signature` on a `message` has been made from the private key behind `address`
/// signature made must be recoverable
/// compatible with bitcoin core `verifymessage`
fn verify_message(address: &str, signature: &str, message: &str) -> Result<bool> {
    let secp = Secp256k1::verification_only();
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

    let address = Address::from_str(&address)?;
    let restored = Address::p2pkh(&pubkey, address.network);

    Ok(address == restored)
}

// json contains signature, the address and descriptor of the address!
#[cfg(test)]
mod tests {
    use crate::offline::import_wallet::{sign_message, verify_message};

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
