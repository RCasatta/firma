use crate::*;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{DerivationPath, ExtendedPubKey};
use bitcoin::Network;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

type BitcoinDescriptor = miniscript::Descriptor<bitcoin::PublicKey>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveAddressOpts {
    pub descriptor: String,
    pub index: u32,
}

/// derive address from descriptor in the form "wsh(multi({n},{x}/{c}/*,{y}/{c}/*,...))#5wstxmwd"
/// NOTE this is an hack waiting miniscript support xpubs in descriptor
pub fn derive_address(network: Network, opt: &DeriveAddressOpts) -> Result<GetAddressOutput> {
    let xpubs = extract_xpubs(&opt.descriptor)?;
    let int_or_ext = extract_int_or_ext(&opt.descriptor)?;
    let path = DerivationPath::from_str(&format!("m/{}/{}", int_or_ext, opt.index))?;
    let secp = Secp256k1::verification_only();
    let pubs: Vec<String> = xpubs
        .iter()
        .filter_map(|x| ExtendedPubKey::from_str(x).ok())
        .filter_map(|x| x.derive_pub(&secp, &path).ok())
        .map(|x| x.public_key.to_string())
        .collect();
    if pubs.len() != xpubs.len() {
        return Err("cannot convert all xpubs to pubs".into());
    }
    let n = extract_n(&opt.descriptor)?;
    let descriptor_with_pubkey = format!("wsh(multi({},{}))", n, pubs.join(","));
    let my_descriptor = BitcoinDescriptor::from_str(&descriptor_with_pubkey[..])?;
    let address = my_descriptor
        .address(network)
        .ok_or_else(|| Error::AddressFromDescriptorFails)?;

    Ok(GetAddressOutput { address, path })
}

/// extract the xpubs from a descriptor in the form "wsh(multi({n},{x}/0/*,{y}/0/*,...))#5wstxmwd"
fn extract_xpubs(descriptor: &str) -> Result<Vec<String>> {
    let mut xpubs = vec![];
    let re = Regex::new("[t|x]pub[1-9A-HJ-NP-Za-km-z]*")?;
    for cap in re.captures_iter(&descriptor) {
        xpubs.push(
            cap.get(0)
                .ok_or_else(|| Error::CaptureGroupNotFound("xpubs".into()))?
                .as_str()
                .to_string(),
        );
    }
    Ok(xpubs)
}

/// extract the n threshold from a descriptor in the form "wsh(multi({n},{x}/0/*,{y}/0/*,...))#5wstxmwd"
fn extract_n(descriptor: &str) -> Result<u8> {
    let err = Error::CaptureGroupNotFound("threshold".into());
    let re = Regex::new("wsh\\(multi\\(([1-9]),")?;
    for cap in re.captures_iter(&descriptor) {
        return Ok(cap.get(1).ok_or(err)?.as_str().parse()?);
    }
    Err(err)
}

/// extract the c index (internal or external) from a descriptor in the form "wsh(multi({n},{x}/{c}/*,{y}/{c}/*,...))#5wstxmwd"
fn extract_int_or_ext(descriptor: &str) -> Result<u8> {
    let err = Error::CaptureGroupNotFound("index".into());
    let re = Regex::new("[t|x]pub[1-9A-HJ-NP-Za-km-z]*/([0-9])/\\*")?;
    for cap in re.captures_iter(&descriptor) {
        return Ok(cap.get(1).ok_or(err)?.as_str().parse()?);
    }
    Err(err)
}

#[cfg(test)]
mod tests {
    use crate::offline::descriptor::*;
    use bitcoin::Network;

    const DESCRIPTOR: &str = "wsh(multi(2,tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU/0/*,tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN/0/*))#5wstxmwd";

    #[test]
    fn extract_xpubs_test() {
        let a ="tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU";
        let b = "tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN";
        let expected = [a, b].to_vec();
        let xpubs = extract_xpubs(&DESCRIPTOR).unwrap();
        assert_eq!(expected, xpubs);
    }

    #[test]
    fn extract_n_test() {
        assert_eq!(2, extract_n(DESCRIPTOR).unwrap());
    }

    #[test]
    fn extract_int_or_ext_test() {
        assert_eq!(0, extract_int_or_ext(DESCRIPTOR).unwrap());
    }

    #[test]
    fn derive_address_test() {
        // firma-online --wallet-name firma-wallet2 get-address --index 0
        // tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk
        let mut opts = DeriveAddressOpts {
            descriptor: DESCRIPTOR.to_string(),
            index: 0,
        };
        let derived_address = derive_address(Network::Testnet, &opts).unwrap();
        assert_eq!(
            "tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk",
            derived_address.address.to_string()
        );
        assert_eq!("m/0/0", derived_address.path.to_string());
        opts.index = 2147483648;
        assert_eq!(
            derive_address(Network::Testnet, &opts)
                .unwrap_err()
                .to_string(),
            "InvalidChildNumber(2147483648)"
        );
    }
}
