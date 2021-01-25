use crate::offline::descriptor::extract_xpubs;
use crate::*;

/// Import a json wallet, used in firma-offline to import existing wallet json descriptor
//TODO android-only at the moment, add support also from command line
//TODO should be generic import
pub fn import_wallet(context: Context, wallet: &WalletJson) -> Result<()> {
    extract_xpubs(&wallet.descriptor)?
        .iter()
        .try_for_each(|xpub| check_compatibility(context.network, xpub.network))?;

    context.write(wallet)?;
    Ok(())
}
