# Firma

Firma is a [psbt](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) (Partially Signed Bitcoin Transaction) [signer](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki#signer). 
It includes tools to create a wallet offline and sign transaction offline.
 

## Requirements

to build executables you need [rust](https://www.rust-lang.org/).

```
cargo build
```

launch tests

```
cargo test
```

## Create Master Key (optional)

This step  create a master key using a dice to provide randomness.
You can skip this step if you already have a master key (`xpub...`) or you want to generate it in another way.

```bash
$ cargo run --bin dice -- --faces 6
Creating Master Private Key for testnet with a 6-sided dice
Need 99 dice launches
1st of 99 launch [1-6]: 3
2nd of 99 launch [1-6]: 5

...
99th of 99 launch [1-6]: 4

key saved in "master_key"

```


```json
{
  "xpub": "tpubD6NzVbkrYhZ4WtVRq6TT4iVQuSB7xVXw2CxCJSATRUuEXE98HGFvTR5QA6d6NCLzFd4rH8jUz4wyWmXXNCVq1czTqB6p5J54EneWbctQcTs",
  "xpriv": "tprv8ZgxMBicQKsPdRTdwSnrfJqJLQfBoAM2SuMR1v8A1D6qgjtMesSLGvTXywBQ5NHqu7JXmVwEWNvrATHf3XhDkr1qF1XMMxSJFuCdDzQSLn6",
  "launches": "[3, 5, 3, 1, 6, 4, 2, 5, 1, 4, 3, 5, 2, 5, 6, 3, 4, 1, 5, 6, 3, 3, 5, 5, 5, 6, 6, 6, 1, 5, 1, 5, 2, 3, 5, 2, 2, 1, 6, 5, 3, 4, 5, 6, 1, 2, 6, 3, 4, 2, 1, 4, 5, 5, 5, 5, 6, 4, 4, 4, 3, 3, 2, 1, 6, 5, 5, 4, 3, 2, 1, 6, 4, 3, 5, 5, 2, 1, 1, 6, 6, 1, 3, 5, 6, 5, 4, 1, 2, 3, 4, 5, 6, 4, 3, 4, 5, 4, 4]",
  "faces": 6
}
```

# p2wpkh

## Import watch-only

Create a watch-only wallet with Bitcoin Core 0.18.1 using p2wpkh.

Create a descriptor with checksum for main addresses and changes, also set a wallet name and the network we are working on:
```bash
$ XPUB=$(cat master_key | jq -r .xpub)
$ MAIN=$(bitcoin-cli -testnet getdescriptorinfo "wpkh(${XPUB}/0/*)" | jq -r .descriptor)
$ CHANGE=$(bitcoin-cli -testnet getdescriptorinfo "wpkh(${XPUB}/1/*)" | jq -r .descriptor)
$ WALLET=firma
$ NETWORK=testnet
```

Create a new wallet "firma" (with private key disabled) and import the previously created descriptors (note: rescan is false because we are generating a new wallet, set it to true to import used wallet)
```bash
$ bitcoin-cli -${NETWORK} createwallet "${WALLET}" true
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} importmulti '[{"desc": "'${MAIN}'", "internal": false, "range": [0, 1000], "timestamp": "now", "keypool": true, "watchonly": true}, {"desc": "'${CHANGE}'", "internal": true,  "range": [0, 1000], "timestamp": "now", "keypool": true, "watchonly": true}]' '{ "rescan": false}'
```
Note: even with rescan equal to false, `importmulti` takes a while

## Create PSBT

Create a new address from the just created wallet
```bash
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} getnewaddress
tb1q9ajjavgkqk0j9n6a5pfq736qad37avym8ezalu
```

Fund the address with some testnet bitcoin, then create the psbt.
Since funds are watchonly we must specify input to use through `listunspent`. Sending 0.0012345 to tb1qrxye2d9e5qgsg0qd647rl7drs8p4ytzlylr2ggceppd4djj58gws84d0gv.
We put the result in `psbt.txt`
```bash
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} walletcreatefundedpsbt '[]' '[{"tb1qrxye2d9e5qgsg0qd647rl7drs8p4ytzlylr2ggceppd4djj58gws84d0gv":0.0012345}]' 0 '{"includeWatching":true}' true> psbt.txt
```

```json
{
  "psbt": "cHNidP8BAH0CAAAAARN2KBB/eGQV9dqjIWGL7Q14lpqRJS8cH0+sY1+WArlCAAAAAAD/////Ao2tDQAAAAAAFgAU+RTskSlK3nzD/6TiWHH7AynbriQ64gEAAAAAACIAIBmJlTS5oBEEPA3VfD/5o4HDUixfJ8akIxkIW1bKVDodAAAAAAABAR9gkA8AAAAAABYAFC9lLrEWBZ8iz12gUg9HQOtj7rCbAAAA",
  "fee": 0.00000153,
  "changepos": 0
}
```

## Sign PSBT

```bash
$ cargo run -- --key master_key psbt.txt 

inputs [# prevout:vout value]:
#0 23dc82a9c716461f976ce89ce5c0519c87ffd62ce4be5804a0f75d16421a04d1:1 246464

outputs [# script address amount]:
#0 002019899534b9a011043c0dd57c3ff9a381c3522c5f27c6a42319085b56ca543a1d tb1qrxye2d9e5qgsg0qd647rl7drs8p4ytzlylr2ggceppd4djj58gws84d0gv 123450
#1 00140c3e2a4e0911aac188fe1cba6ef3d808326e6d0a tb1qpslz5nsfzx4vrz87rjaxau7cpqexumg2dhryka 122861

absolute fee      :    153 satoshi
unsigned tx       :    125 vbyte
unsigned fee rate :      1 sat/vbyte

Added signatures, wrote "psbt.txt"

$ SIGNED_PSBT=$(cat psbt.txt | jq -r .signed_psbt)
```

## Send PSBT

```bash
$ TX=$(bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} finalizepsbt $SIGNED_PSBT | jq -r .hex)
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} sendrawtransaction $TX
e0b4ba5736f6795d69267bd10db979805bdc97ee10257b6d42b954dbc90d06c0
```

# p2wsh (2of2 multisig)

Supposing we have `master_key` from p2wpkh and `master_key_2` as following

```json
{
  "xpub": "tpubD6NzVbkrYhZ4Wc77iw2W3C5EfGsHkR6TXGoVwBSoUZjVj3hdZ4bNF8eskirtD98DKcNoT3gjKcmiBxpsZX1yV3aaN6rUaM7UhoRZ85kHqwY",
  "xpriv": "tprv8ZgxMBicQKsPd95KqHMudnR86FMMb5uYwyCiefQW4Hw6tZSrvfmn4e31abDadoRxm11yDtPtcThCegUmYeQrdupLHJ9nEj7UPKhxBcrjYYL",
  "launches": "[5, 3, 5, 6, 1, 2, 2, 3, 3, 4, 2, 1, 6, 3, 2, 4, 3, 2, 2, 5, 6, 6, 2, 2, 3, 3, 5, 3, 4, 3, 1, 1, 2, 1, 2, 5, 3, 6, 5, 4, 2, 3, 3, 6, 1, 6, 5, 5, 3, 3, 2, 2, 1, 5, 4, 4, 4, 5, 6, 3, 3, 2, 1, 2, 2, 2, 4, 4, 5, 3, 6, 3, 3, 2, 1, 2, 4, 4, 2, 3, 5, 2, 3, 4, 1, 5, 3, 4, 1, 6, 5, 4, 1, 5, 2, 3, 3, 4, 1]",
  "faces": 6
}
```

```bash
$ XPUB1=$(cat master_key | jq -r .xpub)
$ XPUB2=$(cat master_key_2 | jq -r .xpub)
$ MAIN=$(bitcoin-cli -testnet getdescriptorinfo "wsh(multi(2,${XPUB1}/0/*,${XPUB2}/0/*))" | jq -r .descriptor)
$ CHANGE=$(bitcoin-cli -testnet getdescriptorinfo "wsh(multi(2,${XPUB1}/1/*,${XPUB2}/1/*))" | jq -r .descriptor)
$ WALLET=multifirma
$ NETWORK=testnet
```

```bash
$ bitcoin-cli -${NETWORK} createwallet "${WALLET}" true 
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} importmulti '[{"desc": "'${MAIN}'", "internal": false, "range": [0, 1000], "timestamp": "now", "keypool": true, "watchonly": true}, {"desc": "'${CHANGE}'", "internal": true,  "range": [0, 1000], "timestamp": "now", "keypool": true, "watchonly": true}]' '{ "rescan": false}'

```

Note: even with rescan equal to false, `importmulti` takes a while

Create a new address from the just created wallet, we can't use `getewaddress` because multisig addresses are not yet handled by the keypool.
We also explicitly create a change address for similar reason.

```bash
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} deriveaddresses ${MAIN} 0 | jq -r '.[]'
tb1qp99u5ue2qs2ttthpqpjhtc0qhf6r47g0vtl60cvw52lrtfe7gllqauuj49
$ bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} deriveaddresses ${CHANGE} 0 | jq -r '.[]'
tb1qmkzvhdr23alghczwyaj0p2zxvs73ysxene09c53yl0ven2xfwc5q82artm
```

```bash
bitcoin-cli -${NETWORK} -rpcwallet=${WALLET} walletcreatefundedpsbt '[]' '[{"tb1qrxye2d9e5qgsg0qd647rl7drs8p4ytzlylr2ggceppd4djj58gws84d0gv":0.0012345}]' 0 '{"includeWatching":true, "changeAddress":"tb1qmkzvhdr23alghczwyaj0p2zxvs73ysxene09c53yl0ven2xfwc5q82artm"}' true> psbt_2.txt

```


TODO



