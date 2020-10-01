use crate::*;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPubKey};
use bitcoin::Network;
use miniscript::descriptor::DescriptorPublicKey;
use miniscript::Descriptor;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveAddressOpts {
    pub descriptor: String,
    pub index: u32,
}

/// derive address from descriptor in the form "wsh(multi({n},{x}/{c}/*,{y}/{c}/*,...))#5wstxmwd"
pub fn derive_address(
    network: Network,
    opt: &DeriveAddressOpts,
    int_or_ext: u32,
) -> Result<GetAddressOutput> {
    // checksum not supported at the moment, stripping out
    let end = opt
        .descriptor
        .find('#')
        .unwrap_or_else(|| opt.descriptor.len());
    let descriptor: miniscript::Descriptor<DescriptorPublicKey> = opt.descriptor[..end].parse()?;

    let address = descriptor
        .derive(ChildNumber::from_normal_idx(opt.index)?)
        .address(network)
        .ok_or_else(|| Error::AddressFromDescriptorFails)?;
    let path = DerivationPath::from_str(&format!("m/{}/{}", int_or_ext, opt.index))?;

    Ok(GetAddressOutput { address, path })
}

/// extract the xpubs from a descriptor in the form "wsh(multi({n},{x}/0/*,{y}/0/*,...))#5wstxmwd"
pub fn extract_xpubs(descriptor: &str) -> Result<Vec<ExtendedPubKey>> {
    let mut xpubs = vec![];
    let end = descriptor.find('#').unwrap_or_else(|| descriptor.len());
    let descriptor: miniscript::Descriptor<DescriptorPublicKey> =
        descriptor[..end].parse().unwrap();
    if let Descriptor::Wsh(miniscript) = descriptor {
        for el in miniscript.get_leaf_pk() {
            if let DescriptorPublicKey::XPub(desc_xpub) = el {
                xpubs.push(desc_xpub.xpub);
            }
        }
    }
    Ok(xpubs)
}

#[cfg(test)]
mod tests {
    use crate::offline::descriptor::*;
    use bitcoin::util::bip32::ExtendedPubKey;
    use bitcoin::Network;
    use std::str::FromStr;

    const DESCRIPTOR: &str = "wsh(multi(2,tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU/0/*,tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN/0/*))#5wstxmwd";

    #[test]
    fn derive_address_test() {
        // firma-online --wallet-name firma-wallet2 get-address --index 0
        // tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk
        let mut opts = DeriveAddressOpts {
            descriptor: DESCRIPTOR.to_string(),
            index: 0,
        };
        let derived_address = derive_address(Network::Testnet, &opts, 0).unwrap();

        assert_eq!(
            "tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk",
            derived_address.address.to_string()
        );

        assert_eq!("m/0/0", derived_address.path.to_string());
        opts.index = 2147483648;
        assert_eq!(
            derive_address(Network::Testnet, &opts, 0)
                .unwrap_err()
                .to_string(),
            "InvalidChildNumber(2147483648)"
        );
    }

    #[test]
    fn extract_xpubs_test() {
        let a = ExtendedPubKey::from_str("tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU").unwrap();
        let b = ExtendedPubKey::from_str("tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN").unwrap();
        let expected = [a, b].to_vec();
        let xpubs = extract_xpubs(&DESCRIPTOR).unwrap();
        assert_eq!(expected, xpubs);
    }
}
