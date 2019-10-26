use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use firma::MasterKeyJson;
use num_bigint::BigUint;
use secp256k1::constants::CURVE_ORDER;
use secp256k1::Secp256k1;
use std::fs;
use std::io::{self, BufRead, Lines, StdinLock, Write};
use std::path::PathBuf;
use structopt::StructOpt;

/// Dice generate a bitcoin master key in bip32
#[derive(StructOpt, Debug)]
#[structopt(name = "dice")]
struct Opt {
    /// Network (bitcoin, testnet, regtest)
    #[structopt(short, long, default_value = "testnet")]
    network: Network,

    /// File where to output the master key (xpriv...)
    #[structopt(short, long, parse(from_os_str), default_value = "master_key")]
    output: PathBuf,

    /// Number of faces of the dice
    #[structopt(short, long)]
    faces: u32,
}

fn n() -> BigUint {
    BigUint::from_bytes_be(&CURVE_ORDER[..])
}

pub fn main() {
    let opt = Opt::from_args();
    if opt.output.exists() {
        println!(
            "Output file {:?} exists, exiting to avoid unwanted override. Run --help.",
            &opt.output
        );
        return;
    }
    println!(
        "Creating Master Private Key for {} with a {}-sided dice",
        opt.network, opt.faces
    );

    let count: u32 = required_dice_launches(opt.faces, &n());
    println!("Need {} dice launches", count);

    let launches: Vec<u32> = ask_launches(count, opt.faces);
    println!("Launches: {:?}", launches);

    let master_key = calculate_key(&launches, opt.faces, opt.network);
    println!("{:#?}", master_key);

    let filename = save(&master_key, &opt.output);
    println!("key saved in {}", filename);
}

fn save(master_key: &MasterKeyJson, output: &PathBuf) -> String {
    fs::write(output, serde_json::to_string_pretty(master_key).unwrap())
        .unwrap_or_else(|_| panic!("Unable to write {:?}", output));

    format!("{:?}", output)
}

fn ask_launches(count: u32, faces: u32) -> Vec<u32> {
    let mut launches = vec![];
    for i in 1..=count {
        let question = format!("{}{} of {} launch", i, endish(i), count);
        let val = ask_number(&question, 1, faces);
        launches.push(val);
    }
    launches
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

fn calculate_key(launches: &[u32], faces: u32, network: Network) -> MasterKeyJson {
    let acc = multiply_dice_launches(&launches, faces);

    assert!(acc < n());

    let sec = acc.to_bytes_be();
    let secp = Secp256k1::signing_only();

    let xpriv = ExtendedPrivKey::new_master(network, &sec).unwrap();
    let xpub = ExtendedPubKey::from_private(&secp, &xpriv);

    MasterKeyJson {
        xpriv: xpriv.to_string(),
        xpub: xpub.to_string(),
        faces,
        launches: format!("{:?}", launches), // ugly, using a string to avoid going newline for every element
    }
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

fn ask_number(question: &str, min: u32, max: u32) -> u32 {
    let stdin = io::stdin();
    let mut stdin: Lines<StdinLock> = stdin.lock().lines();
    loop {
        print!("{} [{}-{}]: ", question, min, max);
        io::stdout().flush().unwrap();
        let line = stdin.next().unwrap().unwrap().parse::<u32>();
        if let Ok(val) = line {
            if val >= min && val <= max {
                return val;
            } else {
                println!("Out of range");
            }
        } else {
            println!("Not a number");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use num_bigint::BigUint;

    #[test]
    fn test_required_dice_launches() {
        assert_eq!(required_dice_launches(6, &BigUint::from(5u32)), 0);
        assert_eq!(required_dice_launches(6, &BigUint::from(6u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(7u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(35u32)), 1);
        assert_eq!(required_dice_launches(6, &BigUint::from(36u32)), 2);
        assert_eq!(required_dice_launches(6, &BigUint::from(37u32)), 2);
        assert_eq!(required_dice_launches(256, &BigUint::from(7u32)), 0);
        assert_eq!(required_dice_launches(256, &n()), 31);
        assert_eq!(required_dice_launches(8, &n()), 85);
        assert_eq!(required_dice_launches(6, &n()), 99);

        let n2 = n() * 2u32;
        assert!(BigUint::from(6u32).modpow(&BigUint::from(99u32), &n2) < n());
        assert!(BigUint::from(6u32).modpow(&BigUint::from(100u32), &n2) > n());
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
    fn test_master_from_dice() {
        // priv1.key and priv2.key taken from https://github.com/tyler-smith/go-bip32/blob/master/bip32_test.go

        let bytes = include_bytes!("../../test_data/dice/priv1.key");
        let expected: MasterKeyJson = serde_json::from_slice(bytes).unwrap();
        assert_eq!(calculate_key(&vec![2], 2, Network::Bitcoin), expected);

        let bytes = include_bytes!("../../test_data/dice/priv2.key");
        let expected: MasterKeyJson = serde_json::from_slice(bytes).unwrap();
        assert_eq!(
            calculate_key(&vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 1], 256, Network::Bitcoin),
            expected
        );
    }
}
