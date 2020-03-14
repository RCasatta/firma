use crate::*;
use bitcoin::Address;
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
    pub fn get_address(&self, cmd_index: Option<u32>, is_change: bool) -> Result<Address> {
        let (wallet, mut index_json) = self.context.load_wallet_and_index()?;

        let (index, descriptor) = if is_change {
            (index_json.change, wallet.change_descriptor)
        } else {
            match cmd_index {
                Some(index) => (index, wallet.main_descriptor),
                None => (index_json.main, wallet.main_descriptor),
            }
        };
        let address_type = if is_change { "change" } else { "external" };

        info!("Creating {} address at index {}", address_type, index);
        let addresses = self
            .client
            .derive_addresses(&descriptor, Some([index, index]))?;
        let address = &addresses[0];
        if address.network != self.context.network {
            return Err("address returned is not on the same network as given".into());
        }
        info!("{}", address);

        if is_change {
            index_json.change += 1;
            self.context.save_index(&index_json)?;
        } else if cmd_index.is_none() {
            index_json.main += 1;
            self.context.save_index(&index_json)?;
        }

        Ok(address.clone())
    }
}
