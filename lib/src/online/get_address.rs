use crate::*;
use bitcoincore_rpc::RpcApi;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct GetAddressOptions {
    /// Explicitly specify address derivation index (by default taken from .firma and incremented)
    #[structopt(long)]
    pub index: Option<u32>,
}

impl Wallet {
    pub fn get_address(&self, cmd_index: Option<u32>, is_change: bool) -> Result<GetAddressOutput> {
        let (wallet, mut indexes) = self.context.load_wallet_and_index()?;

        let (index, descriptor) = if is_change {
            (indexes.change, wallet.descriptor_change)
        } else {
            match cmd_index {
                Some(index) => (index, wallet.descriptor_main),
                None => (indexes.main, wallet.descriptor_main),
            }
        };
        let address_type = if is_change { "change" } else { "external" };

        info!("Creating {} address at index {}", address_type, index);

        let addresses = self
            .client
            .derive_addresses(&descriptor, Some([index, index]))?;
        //TODO derive it twice? You know bitflips
        let address = addresses
            .first()
            .ok_or_else(|| Error::MissingAddress)?
            .clone();
        if address.network != self.context.network {
            return Err("address returned is not on the same network as given".into());
        }
        info!("{}", address);

        let derive_address =
            crate::offline::descriptor::derive_address(&descriptor, index, self.context.network)?;
        assert_eq!(
            derive_address.address, address,
            "address generated from the node differs from the one generated from miniscript"
        );

        if is_change {
            indexes.change += 1;
            self.context.save_index(&indexes)?;
        } else if cmd_index.is_none() {
            indexes.main += 1;
            self.context.save_index(&indexes)?;
        }

        Ok(derive_address)
    }
}
