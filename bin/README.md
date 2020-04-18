
NOTE: written for firma 0.1.0 some command and/or output could be slightly different

# Creating a 2of2 multisig wallet p2wsh

During the following step we are going to create a multisig wallet 2of2 in testnet and we are going to sign and broadcast a transaction. 
It is required a synced bitcoin node.

In the first steps we are going to create two master keys.

## Create first Master Key

This step creates the first master key. (It is possible to create the key with dice, see `firma-online dice --help`)

```
firma-offline random --key-name r1
```
```json
{
  "key": {
    "xprv": "tprv8ZgxMBicQKsPfBh8CR3miH6A3pEi1Egu31m3PzmfqnuxCqhGqu3LcE3PAtxSFmfospHiANXrKse8HTHQgNCcb9ntyFwDiPJ1E4VFHFNyYar",
    "xpub": "tpubD6NzVbkrYhZ4Yeiv64iN7gkGcqkeAZsocKMpgWoyG4iM3Kx3UHrvnifFM4mxCm9hpR22pcrSB3HLuhJsVt7xgBAgAE5NRZdWbt7gHTNLZWK"
  },
  "private_file": "/Users/casatta/.firma/testnet/r1-PRIVATE.json",
  "public_file": "/Users/casatta/.firma/testnet/r1-public.json"
}
```

## Create second Master Key

```
firma-offline random --key-name r2
```
```json
{
  "key": {
    "xprv": "tprv8ZgxMBicQKsPeX1gcQSpJ9XYAaRvJjZnthJ7uHKqUYMyXd78S44TS24aJbDALQ1KPNjNHHi6Yn8AAQ9ccC1WyAPWxwfHiYJQ7EVjsu3y8BU",
    "xpub": "tpubD6NzVbkrYhZ4Xz3UW47QhZBejbwrU4khTztuBoN8tpANN7Mu4St3cWgSUkrZc8v9FbFZaLwCDPHo8gKW3R1GqNTADCSrHpGkAVMyEKUbz4q"
  },
  "private_file": "/Users/casatta/.firma/testnet/r2-PRIVATE.json",
  "public_file": "/Users/casatta/.firma/testnet/r2-public.json"
}
```

# Create the 2of2 multisig wallet

For the example we are using the two master_key created in the previous step. From the offline machines 
copy `$HOME/.firma/testnet/r1-public.json` and `$HOME/.firma/testnet/r2-public.json` to the 
online machine. 
`COOKIE_FILE` must point to the bitcoin node cookie file

```
firma-online --wallet-name firma-wallet create-wallet --url http://127.0.0.1:18332 --cookie-file $COOKIE_FILE -r 2 --xpub $HOME/.firma/testnet/r1-public.json --xpub $HOME/.firma/testnet/r2-public.json
```
```json
{
     "wallet": {
       "daemon_opts": {
         "cookie_file": "~/.bitcoin/testnet3/.cookie",
         "url": "http://127.0.0.1:18332"
       },
       "descriptor_change": "wsh(multi(2,tpubD6NzVbkrYhZ4Yeiv64iN7gkGcqkeAZsocKMpgWoyG4iM3Kx3UHrvnifFM4mxCm9hpR22pcrSB3HLuhJsVt7xgBAgAE5NRZdWbt7gHTNLZWK/1/*,tpubD6NzVbkrYhZ4Xz3UW47QhZBejbwrU4khTztuBoN8tpANN7Mu4St3cWgSUkrZc8v9FbFZaLwCDPHo8gKW3R1GqNTADCSrHpGkAVMyEKUbz4q/1/*))#4rapk0uk",
       "descriptor_main": "wsh(multi(2,tpubD6NzVbkrYhZ4Yeiv64iN7gkGcqkeAZsocKMpgWoyG4iM3Kx3UHrvnifFM4mxCm9hpR22pcrSB3HLuhJsVt7xgBAgAE5NRZdWbt7gHTNLZWK/0/*,tpubD6NzVbkrYhZ4Xz3UW47QhZBejbwrU4khTztuBoN8tpANN7Mu4St3cWgSUkrZc8v9FbFZaLwCDPHo8gKW3R1GqNTADCSrHpGkAVMyEKUbz4q/0/*))#krd5ntg9",
       "name": "firma-wallet"
     },
     "wallet_file": "/Users/casatta/.firma/testnet/firma-wallet/descriptor.json"
   }
```

## Create a receiving address

Create a new address from the just generated wallet. Bitcoin node parameters are not needed anymore since have been saved in `$HOME/.firma/testnet/firma-wallet/descriptor.json`

```
firma-online --wallet-name firma-wallet get-address
```
```json
{
  "address": "tb1qqnldnf79cav7mu9f36f9r667mgucltzyr3ht0h2j67nfwtyvz4qscfwkzv",
  "indexes": {
    "change": 0,
    "main": 1
  }
}
```

Send some funds to `tb1qqnldnf79cav7mu9f36f9r667mgucltzyr3ht0h2j67nfwtyvz4qscfwkzv`

## Check balance and coins

```
firma-online --wallet-name firma-wallet balance
```
```json
{
  "btc": "0.00023134",
  "satoshi": 23134
}
```
```
firma-online --wallet-name firma-wallet list-coins 
```
```
{
  "coins": [
    {
      "amount": 23134,
      "outpoint": "232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1"
    }
  ]
}
```

## Create the PSBT

After funds receive a confirmation we can create the PSBT specifiying the recipient and the amount, you can specify more than one recipient and you can explicitly spend specific utxo with `--coin`. See `firma-online create-tx --help`

```
firma-online --wallet-name firma-wallet create-tx --recipient tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x:5234
```
```json
{
  "psbt_file": "~/.firma/psbt-0.json",
  "result": {
    "changepos": 1,
    "fee": 1.93e-6,
    "psbt": "cHNidP8BAIkCAAAAAQVA/hgjlczjqQNXAOrYFB6A1DDi2wKtNeEwqZUbNi0jAQAAAAD+////AnIUAAAAAAAAIgAgF3Xq1BrO+hTS1TTWJy2mEMw1hV0N5Mqw9cGj+JSSGYkrRQAAAAAAACIAIPw+DbBPtPjcpiitaD2hmdgrvsiyIg635s1W5ooKdjCvAAAAAAABASteWgAAAAAAACIAIAT+2afFx1nt8KmOklHrXto5j6xEHG633VLXppcsjBVBAQVHUiEDUXWB4Qh47xCKpto2HMdBsWTsRwYHfqq4zwLbjrkyWXshA6TTyJX+H1fHr5WESfNhUa0y9qF/L0RiXdjriThsre8gUq4iBgNRdYHhCHjvEIqm2jYcx0GxZOxHBgd+qrjPAtuOuTJZewyswi4WAAAAAAAAAAAiBgOk08iV/h9Xx6+VhEnzYVGtMvahfy9EYl3Y64k4bK3vIAw0YJNMAAAAAAAAAAAAAAEBR1IhA4A+BIno9JcanXaxLzE8maixH/PtiqT+l59202amEHK5IQJHbVNsy+rKmbMOmqwYYVl/uVoQN//Is7yAXr2sBckqPlKuIgICR21TbMvqypmzDpqsGGFZf7laEDf/yLO8gF69rAXJKj4MNGCTTAEAAAABAAAAIgIDgD4Eiej0lxqddrEvMTyZqLEf8+2KpP6Xn3bTZqYQcrkMrMIuFgEAAAABAAAAAA=="
  }
}
```

Copy `psbt-0.json` to the two offline nodes as `psbt-0-A.json` and `psbt-0-B.json`.
Copy on the offline nodes also the wallet descriptor `$HOME/.firma/testnet/firma-wallet/descriptor.json`

## Sign from node A

```
firma-offline sign psbt-0-A.json --key $HOME/.firma/testnet/r1-PRIVATE.json
```
```json
{
  "fee": {
    "absolute": 193,
    "rate": 1.0157894736842106
  },
  "info": [
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    "#0 232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1 (m/0/0) 23134"
  ],
  "outputs": [
    "#0 00201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f894921989 tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x () 5234",
    "#1 0020fc3e0db04fb4f8dca628ad683da199d82bbec8b2220eb7e6cd56e68a0a7630af tb1qlslqmvz0knudef3g445rmgvemq4maj9jyg8t0ekd2mng5znkxzhsrnfan6 (m/1/1) 17707"
  ],
  "psbt_file": "/Users/casatta/.firma/psbt-0-A.json",
  "sizes": [
    "unsigned tx        :    137   vbyte",
    "estimated tx       :    190   vbyte"
  ]
}
```

## Sign from node B

```
firma-offline sign psbt-0-B.json --key $HOME/.firma/testnet/r2-PRIVATE.json
```
```json
{
  "fee": {
    "absolute": 193,
    "rate": 1.0157894736842106
  },
  "info": [
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    "#0 232d361b95a930e135ad02dbe230d4801e14d8ea005703a9e3cc952318fe4005:1 (m/0/0) 23134"
  ],
  "outputs": [
    "#0 00201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f894921989 tb1qza6744q6emapf5k4xntzwtdxzrxrtp2aphjv4v84cx3l39yjrxys0cg47x () 5234",
    "#1 0020fc3e0db04fb4f8dca628ad683da199d82bbec8b2220eb7e6cd56e68a0a7630af tb1qlslqmvz0knudef3g445rmgvemq4maj9jyg8t0ekd2mng5znkxzhsrnfan6 (m/1/1) 17707"
  ],
  "psbt_file": "/Users/casatta/.firma/psbt-0-B.json",
  "sizes": [
    "unsigned tx        :    137   vbyte",
    "estimated tx       :    190   vbyte"
  ]
}
```

## Combine, finalize and send TX

```
firma-online --wallet-name firma-wallet send-tx --psbt psbt-0-A.json --psbt psbt-0-B.json --broadcast
```
```
{
  "broadcasted": true,
  "hex": "020000000001010540fe182395cce3a9035700ead8141e80d430e2db02ad35e130a9951b362d230100000000feffffff0272140000000000002200201775ead41acefa14d2d534d6272da610cc35855d0de4cab0f5c1a3f8949219892b45000000000000220020fc3e0db04fb4f8dca628ad683da199d82bbec8b2220eb7e6cd56e68a0a7630af0400483045022100c79cdf100e069614d49c0dd69402c4ebd342d4dac508c092805ecd624231868702200327f903f7050b911487b4926ad5ab3544e04b6b795a8408ce0a9b943ea86ff401483045022100d2ada761237d45096f9dafa60a383cdca393c5b552b3acf94f067f8c4d6a058d02206f0dd20c197326648aa693fd3d27f5b34c4d27ded24881ade1ba603292e821cd0147522103517581e10878ef108aa6da361cc741b164ec4706077eaab8cf02db8eb932597b2103a4d3c895fe1f57c7af958449f36151ad32f6a17f2f44625dd8eb89386cadef2052ae00000000",
  "txid": "7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122"
}
```

View tx [7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122](https://blockstream.info/testnet/tx/7695016ce72c9ec2e13a5892f3ac28904c317c66a348cd1f5407c9128d12b122)


