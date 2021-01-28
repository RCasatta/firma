use crate::offline::descriptor::DeriveAddressOptions;
use crate::*;
use bitcoincore_rpc::RpcApi;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
pub struct GetAddressOptions {
    /// The name of the wallet to be created
    #[structopt(long = "wallet-name")]
    pub wallet_name: String,

    /// Explicitly specify address derivation index (by default taken from .firma and incremented)
    #[structopt(long)]
    pub index: Option<u32>,
}

impl Context {
    pub fn get_address(&self, opt: &GetAddressOptions) -> Result<GetAddressOutput> {
        let client = self.make_client(&opt.wallet_name)?;
        let wallet: WalletJson = self.read(&opt.wallet_name)?;
        let mut indexes: IndexesJson = self.read(&opt.wallet_name)?;

        let index = opt.index.unwrap_or(indexes.main);
        let descriptor = wallet.descriptor;

        info!("Creating address at index {} for {}", index, &descriptor);

        let addresses = client.derive_addresses(&descriptor, Some([index, index]))?;

        let address = addresses.first().ok_or(Error::MissingAddress)?.clone();
        if address.network != self.network {
            return Err("address returned is not on the same network as given".into());
        }
        info!("{}", address);

        let derive_opts = DeriveAddressOptions { descriptor, index };
        let derive_address =
            crate::offline::descriptor::derive_address(self.network, &derive_opts)?;
        assert_eq!(
            derive_address.address, address,
            "address generated from the node differs from the one generated from miniscript"
        );

        indexes.main += 1;
        self.write(&indexes)?;

        Ok(derive_address)
    }
}
