# Firma [![Build Status]][travis]

[Build Status]: https://travis-ci.com/RCasatta/firma.svg?branch=master
[travis]: https://travis-ci.com/github/RCasatta/firma

**WARNING - Early stage software, do not use with real bitcoins.**

Firma is a tool to create bitcoin multisig wallets with private keys stored on offline devices.

The offline device could be a [CLI](bin) terminal or a spare [android](android) phone.

Informations are transferred between devices through QR codes. Since PSBT could become large some kB, more than 1 QR code could be needed, those QRs are chained with qr [structured append](https://segno.readthedocs.io/en/stable/structured-append.html) 

It is based on:
  * [bitcoin core](https://bitcoincore.org/)
  * [psbt](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) (Partially Signed Bitcoin Transaction)
  * [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
  * and other [libs](lib/Cargo.toml)
  
## High level process:

```
                                      +---------------------+
+---------------------+               |+---------------------+
|                     |     xpubs     ||                     |
| online machine      | <------------ ||  offline machines   |
|                     |               ||                     |
| * firma-online      |     PSBT      ||  * firma-offline    |
| * bitcoin node      | ------------> ||  * xprv             |
| * xpubs             | <------------ ||                     |
|                     |               +|                     |
+---------------------+                +---------------------+
```

#### Setup

* Create one ore more extended private keys `xprv` on one or more offline devices.
* Group together corresponding extended public keys `xpub` and import these on a (on-line) Bitcoin core node in watch-only mode.
* Bring back the wallet descriptor with `xpubs` on offline machines. While not strictly necessary for signing, wallet on offline machine act as a backup and as added information (eg check if a change is owned by the wallet)

#### Usage

##### Receiving

* `firma-online` tool could create addresses to receive bitcoins.

##### Spending

* Create the transaction from the `firma-online` tool and export it in PSBT format.
* Bring PSBT to offline devices, check the transaction, if everything looks correct, sign the PSBT with the private key present on the device.
* Bring all the PSBT back to the node which can combine and finalize these as complete transaction (this operation could occur in parallel or serially).

## Requirements

You need [Bitcoin core 0.19.1](https://bitcoincore.org/)

To build executables you need [rust](https://www.rust-lang.org/) (version >= 1.38.0).

```
git clone https://github.com/RCasatta/firma/
cd firma
cargo build --release
export PATH=$PATH:$PWD/target/release/
```

## Tests

Integration tests require an env var pointing to bitcoin core executable (`bitcoind`). 

For example:

```
BITCOIN_EXE_DIR=./bitcoin-0.19.1/bin cargo test
```

## Example

Check the bin [readme](bin/README.md) for an example with CLI 

## Donations

I am the maintainer of one of the OpenTimestamps calendar, you can donate [there](https://finney.calendar.eternitywall.com/) (onchain or lightning) if you want to support this work.
