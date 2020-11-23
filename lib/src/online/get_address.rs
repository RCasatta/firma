use crate::offline::descriptor::DeriveAddressOpts;
use crate::qr::QrMode;
use crate::*;
use bitcoin::util::address::Payload;
use bitcoin::Address;
use bitcoincore_rpc::RpcApi;
use log::info;
use qr_code::QrCode;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
pub struct GetAddressOptions {
    /// Explicitly specify address derivation index (by default taken from .firma and incremented)
    #[structopt(long)]
    pub index: Option<u32>,

    /// If we are asking for an internal address, aka change address
    #[structopt(long)]
    pub is_change: bool,

    /// Show the qr in text mode inside the returned json, note that new line are encoded,
    /// to properly see the qr_code you can pipe the json in jq eg. ` | jq -r .qr_text`
    #[structopt(long, default_value = "none")]
    pub qr_mode: QrMode,
}

impl Wallet {
    pub fn get_address(&self, opts: &GetAddressOptions) -> Result<GetAddressOutput> {
        let (wallet, mut indexes) = self.context.load_wallet_and_index()?;

        let (int_or_ext, index, descriptor) = if opts.is_change {
            (1, indexes.change, wallet.descriptor_change)
        } else {
            match opts.index {
                Some(index) => (0, index, wallet.descriptor_main),
                None => (0, indexes.main, wallet.descriptor_main),
            }
        };
        let address_type = if opts.is_change { "change" } else { "external" };

        info!("Creating {} address at index {}", address_type, index);

        let addresses = self
            .client
            .derive_addresses(&descriptor, Some([index, index]))?;
        //TODO derive it twice? You know bitflips
        let address = addresses.first().ok_or(Error::MissingAddress)?.clone();
        if address.network != self.context.network {
            return Err("address returned is not on the same network as given".into());
        }
        info!("{}", address);

        let derive_opts = DeriveAddressOpts { descriptor, index };
        let mut derive_address = crate::offline::descriptor::derive_address(
            self.context.network,
            &derive_opts,
            int_or_ext,
        )?;
        assert_eq!(
            derive_address.address, address,
            "address generated from the node differs from the one generated from miniscript"
        );

        if opts.is_change {
            indexes.change += 1;
            self.context.save_index(&indexes)?;
        } else if opts.index.is_none() {
            indexes.main += 1;
            self.context.save_index(&indexes)?;
        }

        match opts.qr_mode {
            QrMode::Text { inverted } => {
                let qr = addr_to_qr(&derive_address.address)?;
                let (mut output_file, name) = addr_to_file(&derive_address.address, "txt")?;
                derive_address.qr_file = Some(name);
                output_file.write_all(qr.to_string(inverted, 3).as_bytes())?;
            }
            QrMode::Image => {
                let qr = addr_to_qr(&derive_address.address)?;
                let (output_file, name) = addr_to_file(&derive_address.address, "bmp")?;
                derive_address.qr_file = Some(name);
                qr.to_bmp()
                    .mul(3)?
                    .add_white_border(12)?
                    .write(output_file)?;
            }
            QrMode::None => (),
        }

        Ok(derive_address)
    }
}

fn addr_to_file(address: &Address, ext: &str) -> Result<(File, String)> {
    let name = format!("{}.{}", address.to_string(), ext);
    Ok((File::create(&name)?, name))
}

fn addr_to_qr(address: &Address) -> Result<QrCode> {
    let address_string = address.to_string();
    let qr_string = match &address.payload {
        Payload::WitnessProgram { .. } => address_string.to_uppercase(),
        _ => address_string,
    };
    Ok(qr_code::QrCode::new(qr_string.as_bytes())?)
}
