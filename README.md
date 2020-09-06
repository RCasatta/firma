# Firma

**WARNING - Early-stage software, do not use with real bitcoins.**

Firma is a tool to create bitcoin multisig wallets with private keys stored on offline devices.

The offline device could be a [CLI](bin) terminal or a spare [android](android) phone.

Information is transferred between devices through QR codes. Since PSBT could become large some kB, more than 1 QR code could be needed, those QRs are chained with QR [structured append](https://segno.readthedocs.io/en/stable/structured-append.html) 

It is based on:
  * [bitcoin core](https://bitcoincore.org/)
  * [PSBT](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) (Partially Signed Bitcoin Transaction)
  * [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
  * and other [libs](lib/Cargo.toml)
  
## High-level process:

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
* Group together corresponding extended public keys `xpub` and import these on an on-line Bitcoin core node in watch-only mode.
* Bring back the wallet descriptor with `xpubs` on offline machines. While not strictly necessary for signing, wallet on offline machine act as a backup and as added information (eg check if a change is owned by the wallet)

#### Usage

##### Receiving

* The `firma-online` tool could create addresses to receive bitcoins.

##### Spending

* Create the transaction from the `firma-online` tool and export it in PSBT format.
* Bring PSBT to offline devices, check the transaction, if everything looks correct, sign the PSBT with the private key present on the device.
* Bring all the PSBT back to the node which can combine and finalize these as complete transaction (this operation could occur in parallel or serially).

## Requirements

You need [Bitcoin core 0.20.1](https://bitcoincore.org/)

To build executables you need [rust](https://www.rust-lang.org/) (version >= 1.38.0).

```
git clone https://github.com/RCasatta/firma/
cd firma
cargo build --release
```

executables are `target/release/firma-online` and `target/release/firma-offline`

## Tests

Integration tests require an env var pointing to bitcoin core executable (`bitcoind`). 

For example:

```
BITCOIND_EXE=./bitcoin-0.20.1/bin/bitcoind cargo test
```

## Example

Check the bin [readme](bin/README.md) for an example with CLI 

## Faq

<details>
  <summary>How Firma handle fee bug on segwit inputs?</summary>
  
  Full previous tx is included in the PSBT to check the prevout hash match the previous transaction, causing an error if amounts are changed as the attack requires.
</details>
 
<details>
  <summary>How Firma handle attacks on the online wallet generating receive addresses?</summary>

  The offline app could generate addresses as well. The receive process should take into account both an online and an offline device, checking the receiving address generated matches.  
</details>


<details>
  <summary>How Firma offline know the change address is mine?</summary>

  Firma online stores the full watch-only descriptor of the wallet thus could generate the address given the derivation present in the PSBT, if the address matches it is owned by the wallet.
</details>

<details>
  <summary>How Firma tackle physical attacks on the device?</summary>

  It doesn't. The security is given by the multi-signature scheme, the offline software doesn't use any secure element of the device.
   
</details>

<details>
  <summary>Why do I need the wallet descriptor in the offline device?</summary>

  While the wallet descriptor isn't strictly necessary in the offline, it allows some safety checks like the address checking.
  Most importantly the descriptor is absolutely necessary as a part of the backup, for example in 3of5 scheme, 3 master private keys are not enough to sign transactions because we need 5 master public keys.
  For this reason the flow requires every offline device store also the wallet descriptor containing all the master public keys.
  
</details>


## Donations

I am the maintainer of one of the OpenTimestamps calendar, you can donate [there](https://finney.calendar.eternitywall.com/) (onchain or lightning) if you want to support this work.
