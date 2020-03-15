# Firma

**WARNING - Early stage software, do not use with real bitcoins.**

Firma is a tool to create bitcoin multisig wallet with private keys stored on offline machines.

It is based on:
  * [bitcoin core](https://bitcoincore.org/)
  * [psbt](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) (Partially Signed Bitcoin Transaction)
  * [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
  
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

#### Usage

##### Receiving

* The Bitcoin core node could create addresses to receive bitcoins.

##### Spending

* Create the transaction from the Bitcoin core node and export it in PSBT format.
* Bring PSBT to offline devices, check the transaction, if everything looks correct, sign the PSBT with the `xprv`.
* Bring all the PSBT back to the node which can combine and finalize these as complete transaction.

## Requirements

You need [Bitcoin core 0.19.0.1](https://bitcoincore.org/)

To build executables you need [rust](https://www.rust-lang.org/).

```
git clone https://github.com/RCasatta/firma/
cd firma
cargo build --release
export PATH=$PATH:$PWD/target/release/
```

launch tests

```
cargo test
```

# Creating a 2of2 multisig wallet p2wsh

During the following step we are going to create a multisig wallet 2of2 in testnet and we are going to sign and broadcast a transaction. 
It is required a synced bitcoin node.

In the first steps we are going to create two master keys.

## Create Master Key with dice

This step creates a master key using a dice to provide randomness.

```
firma-offline dice --key-name dice --faces 6
```
```
Creating Master Private Key for testnet with a 6-sided dice, saving in "$HOME/.firma/dice.key.json"
Need 49 dice launches to achieve 128 bits of entropy
1st of 49 launch [1-6]: 
3
2nd of 49 launch [1-6]: 
5
...
49th of 49 launch [1-6]: 
4

Saving "$HOME/.firma/testnet/dice-PRIVATE.json"
Saving "$HOME/.firma/testnet/dice-public.json"
```

```json
{
  "xpub": "tpubD6NzVbkrYhZ4Yeiv64iN7gkGcqkeAZsocKMpgWoyG4iM3Kx3UHrvnifFM4mxCm9hpR22pcrSB3HLuhJsVt7xgBAgAE5NRZdWbt7gHTNLZWK",
  "xpriv": "tprv8ZgxMBicQKsPfBh8CR3miH6A3pEi1Egu31m3PzmfqnuxCqhGqu3LcE3PAtxSFmfospHiANXrKse8HTHQgNCcb9ntyFwDiPJ1E4VFHFNyYar",
  "launches": "[3, 5, 3, 1, 6, 4, 2, 5, 1, 4, 3, 5, 2, 5, 6, 3, 4, 1, 5, 6, 3, 3, 5, 5, 5, 6, 6, 6, 1, 5, 1, 5, 2, 2, 1, 6, 5, 3, 4, 5, 6, 1, 2, 6, 3, 4, 2, 1, 4]",
  "faces": 6
}
```

## Create second Master Key randomly

This step creates a master key using the machine random number generator.

```
firma-offline random --key-name random
```
```
Saving "$HOME/.firma/testnet/random-PRIVATE.json"
Saving "$HOME/.firma/testnet/random-public.json"
```

```json
{
  "xpub": "tpubD6NzVbkrYhZ4Xz3UW47QhZBejbwrU4khTztuBoN8tpANN7Mu4St3cWgSUkrZc8v9FbFZaLwCDPHo8gKW3R1GqNTADCSrHpGkAVMyEKUbz4q",
  "xpriv": "tprv8ZgxMBicQKsPeX1gcQSpJ9XYAaRvJjZnthJ7uHKqUYMyXd78S44TS24aJbDALQ1KPNjNHHi6Yn8AAQ9ccC1WyAPWxwfHiYJQ7EVjsu3y8BU"
}
```

# Create the 2of2 multisig wallet

For the example we are using the two master_key created in the previous step. From the offline machines 
copy `$HOME/.firma/testnet/dice-public.json` and `$HOME/.firma/testnet/random-public.json` to the 
online machine. 
`COOKIE_FILE` must point to the bitcoin node cookie file

```
firma-online --wallet-name firma-wallet create-wallet --url http://127.0.0.1:18332 --cookie-file $COOKIE_FILE -r 2 --xpub $HOME/.firma/testnet/dice-public.json --xpub $HOME/.firma/testnet/random-public.json
```
```
Saving wallet data in "$HOME/.firma/testnet/firma-wallet/descriptor.json"
Saving index data in "$HOME/.firma/testnet/firma-wallet/indexes.json"
```

## Create a receiving address

Create a new address from the just generated wallet. Bitcoin node parameters are not needed anymore since have been saved in `$HOME/.firma/testnet/firma-wallet/descriptor.json`

```
firma-online --wallet-name firma-wallet get-address
```
```
Creating external address at index 0
tb1qqnldnf79cav7mu9f36f9r667mgucltzyr3ht0h2j67nfwtyvz4qscfwkzv
Saving index data in "$HOME/.firma/firma-wallet.indexes.json"
```

Send some funds to `tb1qqnldnf79cav7mu9f36f9r667mgucltzyr3ht0h2j67nfwtyvz4qscfwkzv`

## Check balance and coins

```
firma-online --wallet-name firma-wallet
```
```
0.00023134 BTC
```
```
firma-online --wallet-name list-coins 
```
```
232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1 0.00023134 BTC
```


## Create the PSBT

After funds receive a confirmation we can create the PSBT specifiying the recipient and the amount, you can specify more than one recipient and you can explicitly spend specific utxo with `--coin`. See `firma-online create-tx --help`

```
firma-online --wallet-name firma-wallet create-tx --recipient tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x:5234
```
```
Creating change address at index 1
tb1qlslqmvz0knudef3g445rmgvemq4maj9jyg8t0ekd2mng5znkxzhsrnfan6
Saving index data in "$HOME/.firma/testnet/firma-wallet/indexes.json"
wallet_create_funded_psbt WalletCreateFundedPsbtResult {
    psbt: "cHNidP8BAIkCAAAAAQVA/hgjlczjqQNXAOrYFB6A1DDi2wKtNeEwqZUbNi0jAQAAAAD+////AnIUAAAAAAAAIgAgF3Xq1BrO+hTS1TTWJy2mEMw1hV0N5Mqw9cGj+JSSGYkrRQAAAAAAACIAIPw+DbBPtPjcpiitaD2hmdgrvsiyIg635s1W5ooKdjCvAAAAAAABASteWgAAAAAAACIAIAT+2afFx1nt8KmOklHrXto5j6xEHG633VLXppcsjBVBAQVHUiEDUXWB4Qh47xCKpto2HMdBsWTsRwYHfqq4zwLbjrkyWXshA6TTyJX+H1fHr5WESfNhUa0y9qF/L0RiXdjriThsre8gUq4iBgNRdYHhCHjvEIqm2jYcx0GxZOxHBgd+qrjPAtuOuTJZewyswi4WAAAAAAAAAAAiBgOk08iV/h9Xx6+VhEnzYVGtMvahfy9EYl3Y64k4bK3vIAw0YJNMAAAAAAAAAAAAAAEBR1IhA4A+BIno9JcanXaxLzE8maixH/PtiqT+l59202amEHK5IQJHbVNsy+rKmbMOmqwYYVl/uVoQN//Is7yAXr2sBckqPlKuIgICR21TbMvqypmzDpqsGGFZf7laEDf/yLO8gF69rAXJKj4MNGCTTAEAAAABAAAAIgIDgD4Eiej0lxqddrEvMTyZqLEf8+2KpP6Xn3bTZqYQcrkMrMIuFgEAAAABAAAAAA==",
    fee: Amount(193 satoshi),
    change_position: 1,
}
Saving psbt in "psbt-0.json"
```

Copy `psbt-0.json` to the two offline nodes as `psbt-0-A.json` and `psbt-0-B.json`.
Copy on the offline nodes also the wallet descriptor `$HOME/.firma/testnet/firma-wallet/descriptor.json`

## Sign from node A

```
firma-offline sign psbt-0-A.json --key $HOME/.firma/testnet/dice-PRIVATE.json
```
```
Provided PSBT does not contain HD key paths, trying to deduce them...

inputs [# prevout:vout value]:
#0 232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1 (m/0/0) 23134

outputs [# script address amount]:
#0 00201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f894921989 tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x () 5234
#1 0020fc3e0db04fb4f8dca628ad683da199d82bbec8b2220eb7e6cd56e68a0a7630af tb1qlslqmvz0knudef3g445rmgvemq4maj9jyg8t0ekd2mng5znkxzhsrnfan6 (m/1/1) 17707

absolute fee       :    193 satoshi
unsigned tx        :    137 vbyte
estimated tx       :    190 vbyte
estimated fee rate :      1 sat/vbyte

Added signatures, wrote "psbt.0.A.json"
```

## Sign from node B

```
firma-offline sign psbt-0-B.json --key $HOME/.firma/testnet/random-PRIVATE.json
```
```
Provided PSBT does not contain HD key paths, trying to deduce them...

inputs [# prevout:vout value]:
#0 232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1 (m/0/0) 23134

outputs [# script address amount]:
#0 00201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f894921989 tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x () 5234
#1 0020fc3e0db04fb4f8dca628ad683da199d82bbec8b2220eb7e6cd56e68a0a7630af tb1qlslqmvz0knudef3g445rmgvemq4maj9jyg8t0ekd2mng5znkxzhsrnfan6 (m/1/1) 17707

absolute fee       :    193 satoshi
unsigned tx        :    137 vbyte
estimated tx       :    190 vbyte
estimated fee rate :      1 sat/vbyte

Added signatures, wrote "psbt-0-B.json"
```

## Combine, finalize and send TX

```
firma-online --wallet-name firma-wallet send-tx --psbt psbt-0-A.json --psbt psbt-0-B.json --broadcast
```
```
7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122
```

View tx [7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122](https://blockstream.info/testnet/tx/7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122)


