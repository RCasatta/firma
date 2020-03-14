use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use firma::*;
use log::{debug, info};
use num_bigint::BigUint;
use std::io::{self, BufRead, Lines, StdinLock, Write};
use std::str::FromStr;
use structopt::StructOpt;

/// Dice generate a bitcoin master key in bip32
#[derive(StructOpt, Debug)]
#[structopt(name = "dice")]
pub struct DiceOptions {
    /// Number of faces of the dice
    #[structopt(short, long)]
    faces: u32, // TODO only some dice are regular solid, enforce faces 2,6,8,20

    /// Number of bits of entropy
    #[structopt(short, long, default_value = "128")]
    bits: Bits,

    /// Name of the key
    #[structopt(short, long)]
    key_name: String,
}

#[derive(Debug, Clone)]
enum Bits {
    _128,
    _192,
    _256,
}

pub fn roll(datadir: &str, network: Network, opt: &DiceOptions) -> Result<()> {
    debug!("{:?}", opt);
    let (private_file, public_file) = generate_key_filenames(datadir, network, &opt.key_name)?;

    info!(
        "Creating Master Private Key for {} with a {}-sided dice",
        network, opt.faces,
    );
    let bits = &format!("{:?}", opt.bits)[1..];
    let max: BigUint = opt.bits.clone().into();

    let count: u32 = required_dice_launches(opt.faces, &max);
    info!(
        "Need {} dice launches to achieve {} bits of entropy",
        count, bits
    );

    let launches: Vec<u32> = ask_launches(count, opt.faces)?;

    let master_key = calculate_key(&launches, opt.faces, network)?;
    info!("{}", serde_json::to_string_pretty(&master_key)?);

    save_private(&master_key, &private_file)?;
    save_public(&master_key.into(), &public_file)?;

    Ok(())
}

fn ask_launches(count: u32, faces: u32) -> Result<Vec<u32>> {
    let mut launches = vec![];
    for i in 1..=count {
        let question = format!("{}{} of {} launch", i, endish(i), count);
        let val = ask_number(&question, 1, faces)?;
        launches.push(val);
    }
    Ok(launches)
}

fn multiply_dice_launches(launches: &[u32], base: u32) -> BigUint {
    let init = BigUint::from(launches[0] - 1);
    launches.iter().skip(1).fold(init, |mut sum, i| {
        sum *= base;
        sum += i - 1u32;
        sum
    })
}

fn required_dice_launches(faces: u32, max: &BigUint) -> u32 {
    // calculating the number of dice launches needed for the bigger number lesser than n
    let mut count = 0u32;
    let mut acc = BigUint::from(1u32);
    loop {
        count += 1;
        acc *= faces;
        if acc > *max {
            return count - 1;
        }
    }
}

fn calculate_key(launches: &[u32], faces: u32, network: Network) -> Result<PrivateMasterKeyJson> {
    let acc = multiply_dice_launches(&launches, faces);

    let sec = acc.to_bytes_be();
    let secp = Secp256k1::signing_only();

    let xpriv = ExtendedPrivKey::new_master(network, &sec)?;
    let xpub = ExtendedPubKey::from_private(&secp, &xpriv);

    Ok(PrivateMasterKeyJson {
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
        faces: Some(faces),
        launches: Some(format!("{:?}", launches)), // ugly, using a string to avoid going newline for every element
    })
}

fn endish(i: u32) -> String {
    if i >= 11 && i <= 13 {
        "th"
    } else {
        match i % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
    .to_string()
}

fn ask_number(question: &str, min: u32, max: u32) -> Result<u32> {
    let stdin = io::stdin();
    let mut stdin: Lines<StdinLock> = stdin.lock().lines();
    loop {
        info!("{} [{}-{}]: ", question, min, max);
        io::stdout().flush()?;
        let line = stdin
            .next()
            .ok_or_else(fn_err("stdin empty"))??
            .parse::<u32>();
        if let Ok(val) = line {
            if val >= min && val <= max {
                return Ok(val);
            } else {
                info!("Out of range");
            }
        } else {
            info!("Not a number");
        }
    }
}

impl From<Bits> for BigUint {
    fn from(bits: Bits) -> Self {
        let one = BigUint::from(1u32);
        match bits {
            Bits::_128 => one << 128,
            Bits::_192 => one << 192,
            Bits::_256 => one << 256,
        }
    }
}

impl FromStr for Bits {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "128" => Ok(Bits::_128),
            "192" => Ok(Bits::_192),
            "256" => Ok(Bits::_256),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} not in (128, 192, 256)", s),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::*;

    use bitcoin::Network;
    use firma::PrivateMasterKeyJson;
    use num_bigint::BigUint;

    #[test]
    fn test_bits() -> Result<()> {
        let bits: Bits = "128".parse()?;
        let number: BigUint = bits.into();
        assert_eq!(
            "340282366920938463463374607431768211456",
            format!("{}", number)
        );
        Ok(())
    }

    #[test]
    fn test_required_dice_launches() {
        assert_eq!(required_dice_launches(6, &BigUint::from(5u32)), 0);
        assert_eq!(required_dice_launches(6, &BigUint::from(6u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(7u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(35u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(36u32)), 2);
        assert_eq!(required_dice_launches(6, &BigUint::from(37u32)), 2);
        assert_eq!(required_dice_launches(256, &BigUint::from(7u32)), 0);
        let n: BigUint = Bits::_256.into();
        assert_eq!(required_dice_launches(256, &n), 32);
        assert_eq!(required_dice_launches(8, &n), 85);
        assert_eq!(required_dice_launches(6, &n), 99);
        let n: BigUint = Bits::_128.into();
        assert_eq!(required_dice_launches(256, &n), 16);
        let n: BigUint = Bits::_192.into();
        assert_eq!(required_dice_launches(256, &n), 24);
    }

    #[test]
    fn test_multiply_dice_launches() {
        assert_eq!(multiply_dice_launches(&vec![6, 6], 6), BigUint::from(35u32));
        assert_eq!(multiply_dice_launches(&vec![6], 6), BigUint::from(5u32));
        assert_eq!(
            multiply_dice_launches(&vec![10, 10], 10),
            BigUint::from(99u32)
        );
        assert_eq!(
            multiply_dice_launches(&vec![1, 1, 1], 2),
            BigUint::from(0u32)
        );
        assert_eq!(multiply_dice_launches(&vec![2], 2), BigUint::from(1u32));
    }

    #[test]
    fn test_master_from_dice() -> Result<()> {
        // priv1.key and priv2.key taken from https://github.com/tyler-smith/go-bip32/blob/master/bip32_test.go

        let bytes = include_bytes!("../../test_data/dice/priv1.key");
        let expected: PrivateMasterKeyJson = serde_json::from_slice(bytes)?;
        assert_eq!(calculate_key(&vec![2], 2, Network::Bitcoin)?, expected);

        let bytes = include_bytes!("../../test_data/dice/priv2.key");
        let expected: PrivateMasterKeyJson = serde_json::from_slice(bytes)?;
        assert_eq!(
            calculate_key(&vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 1], 256, Network::Bitcoin)?,
            expected
        );
        Ok(())
    }
}
