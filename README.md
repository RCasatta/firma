# Firma

**WARNING - Early stage software, do not use with real bitcoins.**

Firma is a [psbt](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) (Partially Signed Bitcoin Transaction) [signer](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki#signer).

## High level process:

#### Setup

* Create one ore more extended private keys `xpriv` on one or more off-line devices.
* Group together corresponding extended public keys `xpub` and import these on a (on-line) Bitcoin core node in watch-only mode.

#### Usage

##### Receiving

* The Bitcoin core node could create addresses to receive bitcoins.

##### Spending

* Create the transaction from the Bitcoin core node and export it in PSBT format.
* Bring PSBT to offline devices, check the transaction, if everything looks correct, sign the PSBT with the `xpriv`.
* Bring all the PSBT back to the node which can combine and finalize these as complete transaction.

## TODOs

Project is at early stage of development, to contribute, have a look at the [issues](https://github.com/RCasatta/firma/issues).

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

During the following step we are going to create a multisig wallet 2of2 in testnet and we are going to sign and broadcast a transaction. It is required a synced bitcoin node.
First steps we are going to create the master keys

## Create Master Key with dice

This step creates a master key using a dice to provide randomness.
You can skip this step if you already have a master key (`xpriv and corresponding xpub`) or you want to generate it in another way.

```
$ firma-offline dice --key-name dice--faces 6
Creating Master Private Key for testnet with a 6-sided dice, saving in "$HOME/.firma/dice.key.json"
Need 49 dice launches to achieve 128 bits of entropy
1st of 49 launch [1-6]: 
3
2nd of 49 launch [1-6]: 
5
...
49th of 49 launch [1-6]: 
4

key saved in "$HOME/.firma/dice.key.json"
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
$ firma-offline random --key-name random
key saved in "/Users/casatta/.firma/random.key.json"
```

```json
{
  "xpub": "tpubD6NzVbkrYhZ4XGqKPC1AE2hfEKHTeMfrsjEhE1jUeGX8YWUQDtYreQSfG6DiV6MbyhKHUjG7BxFuYdGbjRyMHG6hbKQ8kS3s4BRmMFFtZdm",
  "xpriv": "tprv8ZgxMBicQKsPdooXVYLZpd3YfHmXV2UxJRduwVhBDziji2DdbVjGTupo5xLKrUEyy6Tx52sFMqt7Vn6j6rZGHt4YaBfVd5DNVFTXyDa34vk"
}
```

# Create the multisig wallet

For the example we have are using the two master_key created in the previous step. 
In this example, for simplicity, we assume everything is on the same machine, which is not the configuration desired for this tool.

```
$ XPUB1=$(cat $HOME/.firma/dice.key.json | jq -r .xpub)
$ XPUB2=$(cat $HOME/.firma/random.key.json | jq -r .xpub)
$ firma-online --url http://127.0.0.1:8332 --rpcuser user --rpcpassword password --wallet-name firma-test create-wallet -r 2 --xpub $XPUB1 --xpub $XPUB2
Saving wallet data in "/Users/casatta/.firma/firma-test.descriptor.json"
```

Create a new address from the just generated wallet

```
$ firma-online --url http://127.0.0.1:18332 --rpcuser user --rpcpassword password --wallet-name firma-test get-address
Creating external address at index 0
tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x
Saving index data in "/Users/casatta/.firma/firma-test.indexes.json"
```

Send some funds to `tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x`

Create the PSBT

```
$ firma-online --url http://127.0.0.1:18332 --rpcuser user --rpcpassword password --wallet-name firma-test create-tx --address 2N3T3fgZrs5AC6JtAviqbUyE5J3tu5qGzyv --amount "5000 sat"
Creating change address at index 0
tb1q8wyxjuqmesqjsycy5pcf08cfg2gpv76z4xywjypnzlt4zy73fq8qe8c366
Saving index data in "/Users/casatta/.firma/firma-test.indexes.json"
wallet_create_funded_psbt WalletCreateFundedPsbtResult {
    psbt: "cHNidP8BAH4CAAAAATc+zuW+joVQNSZWVnzLdWSvvxyAe7/XBwerfYDZdhJYAQAAAAD+////AogTAAAAAAAAF6kUb+2iU5GanSPvfMV1PCoLiE+pFTyHUAUAAAAAAAAiACA7iGlwG8wBKBMEoHCXnwlCkBZ7QqmI6RAzF9dRE9FIDgAAAAAAAQErECcAAAAAAAAiACAXderUGs76FNLVNNYnLaYQzDWFXQ3kyrD1waP4lJIZiQEFR1IhA1F1geEIeO8QiqbaNhzHQbFk7EcGB36quM8C2465Mll7IQKZ/vODMP+p3UJZJkyrp5Jgps2LkWDR7c7cbHRIOYJYt1KuIgYCmf7zgzD/qd1CWSZMq6eSYKbNi5Fg0e3O3Gx0SDmCWLcM+LP4/AAAAAAAAAAAIgYDUXWB4Qh47xCKpto2HMdBsWTsRwYHfqq4zwLbjrkyWXsMrMIuFgAAAAAAAAAAAAABAUdSIQPYrcxrYDoQrJyB50WuOoTTTs6L0jm5i564H0wStX3XDCECZ1doOPsamvlOMeszupHha5mwnN20I0GyUmuzxvuoDD1SriICAmdXaDj7Gpr5TjHrM7qR4WuZsJzdtCNBslJrs8b7qAw9DPiz+PwBAAAAAAAAACICA9itzGtgOhCsnIHnRa46hNNOzovSObmLnrgfTBK1fdcMDKzCLhYBAAAAAAAAAAA=",
    fee: Amount(3640 satoshi),
    change_position: 1,
}
Saving psbt in "psbt.0.json"
```
TODO: the fee is too high, depends from the settings of bitcoin-core
TODO: use a bech32 address as recipient with a non rounded amount

Simulate the distribution of the PSBT to the two nodes.
```
cp psbt.0.json psbt.0.A.json
cp psbt.0.json psbt.0.B.json
```

Sign from node A

```
$ firma-offline sign psbt.0.A.json --wallet-name firma-test --key /Users/casatta/.firma/dice.key.json 
Provided PSBT does not contain HD key paths, trying to deduce them...


inputs [# prevout:vout value]:
#0 581276d9807dab0707d7bf7b801cbfaf6475cb7c5656263550858ebee5ce3e37:1 (m/0/0) 10000

outputs [# script address amount]:
#0 a9146feda253919a9d23ef7cc5753c2a0b884fa9153c87 2N3T3fgZrs5AC6JtAviqbUyE5J3tu5qGzyv () 5000
#1 00203b8869701bcc01281304a070979f094290167b42a988e9103317d75113d1480e tb1q8wyxjuqmesqjsycy5pcf08cfg2gpv76z4xywjypnzlt4zy73fq8qe8c366 (m/1/0) 2807

absolute fee       :   2193 satoshi
unsigned tx        :    126 vbyte
estimated tx       :    179 vbyte
estimated fee rate :     12 sat/vbyte

Added signatures, wrote "psbt.0.A.json"
```

Sign from node B

```
$ firma-offline sign psbt.0.B.json --wallet-name firma-test --key /Users/casatta/.firma/random.key.json 
Provided PSBT does not contain HD key paths, trying to deduce them...


inputs [# prevout:vout value]:
#0 581276d9807dab0707d7bf7b801cbfaf6475cb7c5656263550858ebee5ce3e37:1 (m/0/0) 10000

outputs [# script address amount]:
#0 a9146feda253919a9d23ef7cc5753c2a0b884fa9153c87 2N3T3fgZrs5AC6JtAviqbUyE5J3tu5qGzyv () 5000
#1 00203b8869701bcc01281304a070979f094290167b42a988e9103317d75113d1480e tb1q8wyxjuqmesqjsycy5pcf08cfg2gpv76z4xywjypnzlt4zy73fq8qe8c366 (m/1/0) 2807

absolute fee       :   2193 satoshi
unsigned tx        :    126 vbyte
estimated tx       :    179 vbyte
estimated fee rate :     12 sat/vbyte

Added signatures, wrote "psbt.0.B.json"
```

Combine, finalize and send TX

```
$ firma-online --url http://127.0.0.1:18332 --rpcuser user --rpcpassword password --wallet-name firma-test send-tx --psbt psbt.0.A.json --psbt psbt.0.B.json 
txid 08d18db081dbbbeacfd0482e3b69cc9ee221af14539e5cb0c5d190d9471ebab9
```

View tx [08d18db081dbbbeacfd0482e3b69cc9ee221af14539e5cb0c5d190d9471ebab9](https://blockstream.info/testnet/tx/08d18db081dbbbeacfd0482e3b69cc9ee221af14539e5cb0c5d190d9471ebab9)


