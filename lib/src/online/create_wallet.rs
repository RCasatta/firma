use crate::offline::descriptor::extract_xpubs;
use crate::online::{read_xpubs_files, Wallet};
use crate::*;
use bitcoin::secp256k1::recovery::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::util::misc::signed_msg_hash;
use bitcoin::{Address, Network, PrivateKey, PublicKey};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    ImportMultiOptions, ImportMultiRequest, ImportMultiRescanSince,
};
use bitcoincore_rpc::RpcApi;
use log::debug;
use log::info;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CreateWalletOptions {
    /// number of signatures required
    #[structopt(short)]
    pub r: usize,

    /// Extended Public Keys (xpub) that are composing the wallet, given as String (xprv...)
    #[structopt(long = "xpub")]
    pub xpubs: Vec<ExtendedPubKey>,

    /// Extended Public Keys (xpub) that are composing the wallet, given as a json file
    #[structopt(long = "xpub-file")]
    pub xpub_files: Vec<PathBuf>,

    #[structopt(flatten)]
    pub daemon_opts: DaemonOpts,

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,
}

impl CreateWalletOptions {
    fn validate(&self, network: Network) -> Result<()> {
        if self.r == 0 {
            return Err("required signatures cannot be 0".into());
        }

        if self.r > 15 {
            return Err("required signatures cannot be greater than 15".into());
        }

        if self.r > (self.xpubs.len() + self.xpub_files.len()) {
            return Err("required signatures cannot be greater than the number of xpubs".into());
        }

        let mut xpubs = read_xpubs_files(&self.xpub_files)?;
        xpubs.extend(&self.xpubs);

        for xpub in xpubs.iter() {
            if !(network == xpub.network
                || (network == Network::Regtest && xpub.network == Network::Testnet))
            {
                return Err(format!(
                    "detected xpub of another network (cmd:{}) (xpub:{})",
                    network, xpub.network
                )
                .into());
            }

            if xpubs.iter().filter(|xpub2| *xpub2 == xpub).count() > 1 {
                return Err("Cannot use same xpub twice".into());
            }
        }

        Ok(())
    }
}

impl Wallet {
    pub fn create(
        &self,
        daemon_opts: &DaemonOpts,
        opt: &CreateWalletOptions,
        height: u64,
    ) -> Result<CreateWalletOutput> {
        opt.validate(self.context.network)?;
        debug!("create");

        let mut xpubs = read_xpubs_files(&opt.xpub_files)?;
        xpubs.extend(&opt.xpubs);

        let xpub_paths: Vec<String> = xpubs.iter().map(|xpub| format!("{}/0/*", xpub)).collect();
        let descriptor = format!("wsh(multi({},{}))", opt.r, xpub_paths.join(","));
        let descriptor = self.client.get_descriptor_info(&descriptor)?.descriptor; // adds checksum

        self.client
            .create_wallet(&self.context.wallet_name, Some(true), None, None, None)?;

        let mut multi_request: ImportMultiRequest = Default::default();
        multi_request.range = Some((0, 1000)); //TODO should be a parameter
        multi_request.timestamp = ImportMultiRescanSince::Now;
        multi_request.keypool = Some(true);
        multi_request.watchonly = Some(true);
        multi_request.descriptor = Some(&descriptor);
        multi_request.internal = Some(false);

        let multi_options = ImportMultiOptions {
            rescan: Some(false),
        };

        let import_multi_result = self
            .client
            .import_multi(&[multi_request], Some(&multi_options));
        info!("import_multi_result {:?}", import_multi_result);

        let fingerprints = xpubs.iter().map(|x| x.fingerprint()).collect();

        let wallet = WalletJson {
            name: self.context.wallet_name.to_string(),
            descriptor,
            fingerprints,
            required_sig: opt.r,
            created_at_height: height,
        };
        let indexes = WalletIndexes { main: 0u32 };

        let wallet_file = self.context.save_wallet(&wallet)?;
        self.context.save_index(&indexes)?;
        self.context.save_daemon_opts(&daemon_opts)?;

        let wallet_for_qr = wallet.clone();
        let qr_bytes = serde_json::to_vec(&wallet_for_qr)?;

        let wallet_qr_path = self.context.path_for_wallet_qr()?;
        let qr_files = common::qr::save_qrs(qr_bytes, wallet_qr_path, opt.qr_version)?;

        let create_wallet = CreateWalletOutput {
            qr_files,
            wallet_file,
            wallet,
        };

        Ok(create_wallet)
    }
}

pub fn import_wallet(datadir: &str, network: Network, wallet: &WalletJson) -> Result<()> {
    extract_xpubs(&wallet.descriptor)?
        .iter()
        .map(|xpub| check_compatibility(network, xpub.network))
        .collect::<Result<()>>()?;
    let context = Context {
        firma_datadir: datadir.to_string(),
        network,
        wallet_name: wallet.name.clone(),
    };
    context.save_wallet(&wallet)?;
    let wallet_for_qr = wallet.clone();
    let qr_bytes = serde_json::to_vec(&wallet_for_qr)?;

    let wallet_qr_path = context.path_for_wallet_qr()?;
    common::qr::save_qrs(qr_bytes, wallet_qr_path, 14)?;
    Ok(())
}

#[allow(dead_code)]
/// Sign a `message` with the given `private_key` in wif format
/// compatible with bitcoin core `signmessagewithprivkey`
fn sign_message(private_key: &str, message: &str) -> Result<String> {
    let secp = Secp256k1::signing_only();
    let hash = signed_msg_hash(&message);
    let message = Message::from_slice(&hash[..])?; // Can never panic because it's the right size.
    let private_key = PrivateKey::from_wif(&private_key)?;
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
    use crate::online::create_wallet::{sign_message, verify_message};

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
