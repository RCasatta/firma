use crate::*;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ChildNumber, DerivationPath};
use bitcoin::Network;
use miniscript::{DescriptorTrait, TranslatePk2};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveAddressOptions {
    pub descriptor: String,
    pub index: u32,
}

impl DeriveAddressOptions {
    fn validate(&self) -> Result<()> {
        ChildNumber::from_normal_idx(self.index)?;
        Ok(())
    }
}

/// derive address from descriptor in the form "wsh(multi({n},{x}/{c}/*,{y}/{c}/*,...))#5wstxmwd"
pub fn derive_address(network: Network, opt: &DeriveAddressOptions) -> Result<GetAddressOutput> {
    opt.validate()?;
    // checksum not supported at the moment, stripping out
    let end = opt
        .descriptor
        .find('#')
        .unwrap_or_else(|| opt.descriptor.len());
    let descriptor: miniscript::Descriptor<miniscript::DescriptorPublicKey> =
        opt.descriptor[..end].parse()?;

    let secp = Secp256k1::verification_only();
    //let context = DescriptorPublicKeyCtx::new(&secp, ChildNumber::from_normal_idx(opt.index)?);
    let address = descriptor
        .derive(opt.index)
        .translate_pk2(|xpk| xpk.derive_public_key(&secp))
        .unwrap()
        .address(network)?;
    /*let address = descriptor
    .address(network, context)
    .ok_or(Error::AddressFromDescriptorFails)?;*/
    let path = DerivationPath::from_str(&format!("m/0/{}", opt.index))?;

    Ok(GetAddressOutput {
        address,
        path,
        qr_file: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::offline::descriptor::*;
    use bitcoin::Network;

    const DESCRIPTOR: &str = "wsh(multi(2,tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU/0/*,tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN/0/*))#5wstxmwd";

    #[test]
    fn derive_address_test() {
        // firma-online --wallet-name firma-wallet2 get-address --index 0
        // tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk
        let mut opts = DeriveAddressOptions {
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

    #[test]
    fn descriptor_extract_keys_test() {
        let k1 = "[a2ebe04e/48h/1h/0h/2h]tpubDEXDRpvW2srXCSjAvC36zYkSE3jxT1wf7JXDo35Ln4NZpmaMNhq8o9coH9U9BQ5bAN4WDGxXV9d426iYKGorFF5wvv4Wv63cZsCotiXGGkD/0/*";
        let k2 = "[1f5e43d8/48h/1h/0h/2h]tpubDFU4parcXvV8tBYt4rS4a8rGNF1DA32DCnRfhzVL6b3MSiDomV95rv9mb7W7jAPMTohyEYpbhVS8FbmTsuQsFRxDWPJX2ZFEeRPMFz3R1gh/0/*";
        let desc = format!("wsh(multi(2,{},{}))#szg2xsau", k1, k2);
        let wallet = Wallet {
            id: Identifier {
                kind: Kind::Wallet,
                name: "azz".to_string(),
                network: Network::Testnet,
            },
            descriptor: desc.to_string(),
            created_at_height: 0,
        };
        let vec1 = wallet.extract_desc_pub_keys().unwrap();
        assert_eq!(vec1[0].to_string(), k1);
        assert_eq!(vec1[1].to_string(), k2);
        let vec2 = wallet.extract_wallet_sign_keys().unwrap();
        assert_eq!(
            vec2[0].to_string(),
            "027bb5876eac67820017008078dcdfa991549b1222d846f06509285ef3f7469b19"
        );
        assert_eq!(
            vec2[1].to_string(),
            "02a40ceccfd217cea801ccab342e36e0eea57777cad50b879855bc0d1e8fb2a030"
        );
    }
}
