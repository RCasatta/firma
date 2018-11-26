
extern crate bitcoin;
extern crate hex;
extern crate bitcoin_bech32;

use std::env;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::consensus::encode::deserialize;
use std::error::Error;
use bitcoin::Script;
use bitcoin::util::address::Payload;
use bitcoin::util::hash::Hash160;
use bitcoin::Address;
use bitcoin::network::constants::Network;
use bitcoin_bech32::WitnessProgram;
use bitcoin_bech32::u5;

fn main() -> Result<(), Box<Error>> {
    if env::args().len() != 3 {
        println!("Need 2 parameters, psbt and key");
        return Ok(());
    }
    let params : Vec<String> = env::args().collect();
    let psbt = &params[1];
    let _key = &params[2];
    let psbt = hex::decode(psbt)?;
    let psbt : PartiallySignedTransaction = deserialize(&psbt)?;
    println!("{:?}", psbt);
    println!("");
    pretty_print(&psbt);

    Ok(())
}

fn pretty_print(psbt : &PartiallySignedTransaction) {
    let transaction = &psbt.global.unsigned_tx;
    println!("inputs [# prevout vout]:");
    for (i,input) in transaction.input.iter().enumerate() {
        println!("#{} {} {}", i, input.previous_output.txid, input.previous_output.vout);
    }
    println!("");
    println!("outputs [# script address amount]:");
    for (i,output) in transaction.output.iter().enumerate() {
        println!("#{} {} {} {}", i,
                 hex::encode(&output.script_pubkey.as_bytes()),
                 script_to_address(&output.script_pubkey, &Network::Bitcoin).unwrap_or("unknown address".to_string()),
                 output.value
        );
    }
}

fn bech_network (network: &Network) -> bitcoin_bech32::constants::Network {
    match network {
        Network::Bitcoin => bitcoin_bech32::constants::Network::Bitcoin,
        Network::Testnet => bitcoin_bech32::constants::Network::Testnet,
        Network::Regtest => bitcoin_bech32::constants::Network::Regtest,
    }
}

pub fn script_to_address(script: &Script, network: &Network) -> Option<String> {
    let payload = if script.is_p2pkh() {
        Some(Payload::PubkeyHash(Hash160::from(&script[3..23])))
    } else if script.is_p2sh() {
        Some(Payload::ScriptHash(Hash160::from(&script[2..22])))
    } else if script.is_v0_p2wpkh() {
        Some(Payload::WitnessProgram(
            WitnessProgram::new(
                u5::try_from_u8(0).expect("0<32"),
                script[2..22].to_vec(),
                bech_network(network),
            ).unwrap(),
        ))
    } else if script.is_v0_p2wsh() {
        Some(Payload::WitnessProgram(
            WitnessProgram::new(
                u5::try_from_u8(0).expect("0<32"),
                script[2..34].to_vec(),
                bech_network(network),
            ).unwrap(),
        ))
    } else {
        None
    };

    Some(
        Address {
            payload: payload?,
            network: *network,
        }.to_string(),
    )
}
