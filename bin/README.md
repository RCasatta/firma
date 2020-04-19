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
    "fingerprint": "ee72a8ec",
    "name": "r1",
    "seed": {
      "bech32": "ts142ps3glsxq586xrglkhc55fznluw9hta8p436wveeedmuq79yvdqphw8h5",
      "hex": "aa8308a3f030287d1868fdaf8a51229ff8e2dd7d386b1d3999ce5bbe03c5231a",
      "network": "testnet"
    },
    "xprv": "tprv8ZgxMBicQKsPfCEMKKmhRud92Ypg3nWXegEeCNATw3aKvbQz5v4sbm2y9kjVD4YMx3oDZQotvTAjZGjSe7wbEYQTm9iqRRV5jsmwnfATNsu",
    "xpub": "tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU"
  },
  "private_file": "~/.firma/testnet/keys/r1/PRIVATE.json",
  "public_file": "~/.firma/testnet/keys/r1/public.json",
  "public_qr_files": [
    "~/.firma/testnet/keys/r1/qr/qr.png"
  ]
}
```

## Create second Master Key

```
firma-offline random --key-name r2
```
```json
{
  "key": {
    "fingerprint": "7d5d8203",
    "name": "r2",
    "seed": {
      "bech32": "ts130ad4m9r3xs3af8c8n3snacqxk93udhqrzpvwn3ccskkmxh3x54q9fj039",
      "hex": "8bfadaeca389a11ea4f83ce309f700358b1e36e01882c74e38c42d6d9af1352a",
      "network": "testnet"
    },
    "xprv": "tprv8ZgxMBicQKsPdMsqUfg8aqwARoMKtQZRfpApqydPbPepm6vd6pZRxsMEHT8ETmfsko4XCdrUYUX8fRPj2xA3AUq83EqinnVTFcje3YeHocG",
    "xpub": "tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN"
  },
  "private_file": "~/.firma/testnet/keys/r2/PRIVATE.json",
  "public_file": "~/.firma/testnet/keys/r2/public.json",
  "public_qr_files": [
    "~/.firma/testnet/keys/r2/qr/qr.png"
  ]
}
```

# Create the 2of2 multisig wallet

For the example we are using the two master_key created in the previous step. From the offline machines 
copy `$HOME/.firma/testnet/keys/r1/public.json` and `$HOME/.firma/testnet/keys/r2/public.json` to the 
online machine. 
`COOKIE_FILE` must point to the bitcoin node cookie file

```
firma-online --wallet-name firma-wallet create-wallet --url http://127.0.0.1:18332 --cookie-file $COOKIE_FILE -r 2 --xpub-file $HOME/.firma/testnet/keys/r1/public.json --xpub-file $HOME/.firma/testnet/keys/r2/public.json
```
```json
{
  "qr_files": [
    "~/.firma/testnet/wallets/firma-wallet/qr/qr-0.png",
    "~/.firma/testnet/wallets/firma-wallet/qr/qr-1.png"
  ],
  "wallet": {
    "created_at_height": 1720454,
    "daemon_opts": {
      "cookie_file": "/Volumes/Transcend/bitcoin-testnet/testnet3/.cookie",
      "url": "http://127.0.0.1:18332"
    },
    "descriptor_change": "wsh(multi(2,tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU/1/*,tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN/1/*))#hwq7rl67",
    "descriptor_main": "wsh(multi(2,tpubD6NzVbkrYhZ4YfG9CySHqKHFbaLcD7hSDyqRUtCmMKNim5fkiJtTnFeqKsRHMHSK5ddFrhqRr3Ghv1JtuWkBzikuBqKu1xCpjQ9YxoPGgqU/0/*,tpubD6NzVbkrYhZ4WpudNKLizFbGzpsG3jkLF7mc8Vfh1fTDbbBPjDP29My6TaLncaS8VeDPcaNMdUkybucr8Kz9CHSdAtvxnaXyBxPRocefdXN/0/*))#5wstxmwd",
    "fingerprints": [
      "ee72a8ec",
      "7d5d8203"
    ],
    "name": "firma-wallet",
    "required_sig": 2
  },
  "wallet_file": "~/.firma/testnet/wallets/firma-wallet/descriptor.json"
}
```

## Create a receiving address

Create a new address from the just generated wallet. Bitcoin node parameters are not needed anymore since have been saved in `$HOME/.firma/testnet/firma-wallet/descriptor.json`

```
firma-online --wallet-name firma-wallet get-address
```
```json
{
  "address": "tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk",
  "indexes": {
    "change": 0,
    "main": 1
  }
}
```

Send some funds to `tb1q5nrregep899vnvaa5vdpxcwg8794jqy38nu304kl4d7wm4e92yeqz4jfmk`

## Check balance and coins

```
firma-online --wallet-name firma-wallet balance
```
```json
{
  "btc": "0.00002222",
  "satoshi": 2222
}
```
```
firma-online --wallet-name firma-wallet list-coins 
```
```
{
  "coins": [
    {
      "amount": 2222,
      "outpoint": "5a566fb841645d53697cc18a22acf0c7e320fe6501451a815b489b3e056b00e2:1"
    }
  ]
}
```

## Create the PSBT

After funds receive a confirmation we can create the PSBT specifiying the recipient and the amount, you can specify more than one recipient and you can explicitly spend specific utxo with `--coin`. See `firma-online create-tx --help`

```
firma-online --wallet-name firma-wallet create-tx --recipient tb1qcs4rjkn4yplrz3z3065u4u6dxgz4q4qfkx5qaruqn5ppf2k5vajqqd2y2f:1934 --psbt-name test
```
```json
{
  "address_reused": [],
  "funded_psbt": {
    "changepos": -1,
    "fee": 2.88e-6,
    "name": "test",
    "psbt": "cHNidP8BAF4CAAAAAeIAawU+m0hbgRpFAWX+IOPH8KwiisF8aVNdZEG4b1ZaAQAAAAD+////AY4HAAAAAAAAIgAgxCo5WnUgfjFEUX6pyvNNMgVQVAmxqA6PgJ0CFKrUZ2QAAAAAAAEBK64IAAAAAAAAIgAgpMY8oyE5SsmzvaMaE2HIP4tZAJE8+RfW36t87dclUTIBBUdSIQOiEGiE3ON0fGERrkaaG540mWPor9ti8nd1o6+QDkv5wSEC2QMn1YJTDcdOAXorh05UQmPdPO+pZT6L3A0jRrf9WEVSriIGAtkDJ9WCUw3HTgF6K4dOVEJj3TzvqWU+i9wNI0a3/VhFDH1dggMAAAAAAAAAACIGA6IQaITc43R8YRGuRpobnjSZY+iv22Lyd3Wjr5AOS/nBDO5yqOwAAAAAAAAAAAAA"
  },
  "psbt_file": "~/.firma/testnet/psbts/test/psbt.json",
  "qr_files": [
    "~/.firma/testnet/psbts/test/qr/qr-0.png",
    "~/.firma/testnet/psbts/test/qr/qr-1.png"
  ]
}
```

## Sign from node A

```
firma-offline sign ~/.firma/testnet/psbts/test/psbt.json --key $HOME/.firma/testnet/keys/r1/PRIVATE.json --wallet-descriptor-file ~/.firma/testnet/wallets/firma-wallet/descriptor.json
```
```json
{
  "fee": {
    "absolute": 288,
    "rate": 1.9591836734693877
  },
  "info": [
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "5a566fb841645d53697cc18a22acf0c7e320fe6501451a815b489b3e056b00e2:1",
      "path": "m/0/0",
      "value": "0.00002222 BTC",
      "wallet": "firma-wallet"
    }
  ],
  "outputs": [
    {
      "address": "tb1qcs4rjkn4yplrz3z3065u4u6dxgz4q4qfkx5qaruqn5ppf2k5vajqqd2y2f",
      "path": "",
      "value": "0.00001934 BTC",
      "wallet": ""
    }
  ],
  "psbt_file": "~/.firma/testnet/psbts/test-ee72a8ec/psbt.json",
  "size": {
    "estimated": 147,
    "unsigned": 94
  }
}
```

A new psbt.json has been created at `~/.firma/testnet/psbts/test-ee72a8ec/psbt.json` note the name has been postfixed with the fingerprint of the key that signed

## Sign from node B

```
firma-offline sign ~/.firma/testnet/psbts/test/psbt.json --key $HOME/.firma/testnet/keys/r1/PRIVATE.json --wallet-descriptor-file ~/.firma/testnet/wallets/firma-wallet/descriptor.json
```
```json
{
  "fee": {
    "absolute": 288,
    "rate": 1.9591836734693877
  },
  "info": [
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "5a566fb841645d53697cc18a22acf0c7e320fe6501451a815b489b3e056b00e2:1",
      "path": "m/0/0",
      "value": "0.00002222 BTC",
      "wallet": "firma-wallet"
    }
  ],
  "outputs": [
    {
      "address": "tb1qcs4rjkn4yplrz3z3065u4u6dxgz4q4qfkx5qaruqn5ppf2k5vajqqd2y2f",
      "path": "",
      "value": "0.00001934 BTC",
      "wallet": ""
    }
  ],
  "psbt_file": "~/.firma/testnet/psbts/test-ee72a8ec/psbt.json",
  "size": {
    "estimated": 147,
    "unsigned": 94
  }
}
```

## Combine, finalize and send TX

```
firma-online --wallet-name firma-wallet send-tx --psbt-file ~/.firma/testnet/psbts/test-7d5d8203/psbt.json --psbt-file ~/.firma/testnet/psbts/test-ee72a8ec/psbt.json --broadcast```
```
{
  "broadcasted": true,
  "hex": "02000000000101e2006b053e9b485b811a450165fe20e3c7f0ac228ac17c69535d6441b86f565a0100000000feffffff018e07000000000000220020c42a395a75207e3144517ea9caf34d3205505409b1a80e8f809d0214aad467640400483045022100c203ec0585270ace9afa2cf9a10f20365f1c491c693cf47bff3d6d5261492eca0220403f809419d8edefc66fdc0b417b5e86e57f1c2083b8220496c2eaec62b5a4e701483045022100851b1314178ef19fad9c9ef4e2c58b8ee73e31785fad3e779378180e312980190220637779f3959937512c1d403f627e4c0938d4207ad35133a18a1aa5c1a7a23fba0147522103a2106884dce3747c6111ae469a1b9e349963e8afdb62f27775a3af900e4bf9c12102d90327d582530dc74e017a2b874e544263dd3cefa9653e8bdc0d2346b7fd584552ae00000000",
  "txid": "d6d19cad1df29117ad3c9ad0de43d1ab2e64c735212c8c457d7eb0bc8e929711"
}
```

View tx [d6d19cad1df29117ad3c9ad0de43d1ab2e64c735212c8c457d7eb0bc8e929711](https://blockstream.info/testnet/tx/d6d19cad1df29117ad3c9ad0de43d1ab2e64c735212c8c457d7eb0bc8e929711)


