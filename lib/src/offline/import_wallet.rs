use crate::offline::descriptor::extract_xpubs;
use crate::*;
use bitcoin::Network;

/// Import a json wallet, used in firma-offline to import existing wallet json descriptor
//TODO android-only at the moment, add support also from command line
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
    let qr_bytes = serde_json::to_vec(&wallet)?;

    let wallet_qr_path = context.path_for_wallet_qr()?;
    common::qr::save_qrs(qr_bytes, wallet_qr_path, 14)?;
    Ok(())
}
