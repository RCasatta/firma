use crate::*;
use bitcoincore_rpc::RpcApi;
use log::info;
use serde_json::{to_value, Value};
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
        let address = addresses.first().ok_or_else(fn_err("no address"))?.clone();
        if address.network != self.context.network {
            return Err("address returned is not on the same network as given".into());
        }
        info!("{}", address);

        if is_change {
            indexes.change += 1;
            self.context.save_index(&indexes)?;
        } else if cmd_index.is_none() {
            indexes.main += 1;
            self.context.save_index(&indexes)?;
        }
        Ok(GetAddressOutput { address, indexes })
    }

    pub fn get_address_value(&self, cmd_index: Option<u32>, is_change: bool) -> Result<Value> {
        Ok(to_value(self.get_address(cmd_index, is_change)?)?)
    }
}
