# Creating a 2of2 multisig wallet p2wsh

During the following step we are going to create a multisig wallet 2of2 in testnet and we are going to sign and broadcast a transaction. 
It is required a synced bitcoin node.

In the first steps we are going to create two master keys.

## Create first Master Key

This step creates the first master key. (It is possible to create the key with dice, see `firma-offline dice --help`)

```
firma-offline random --key-name a1
```
```json
{
  "key": {
    "fingerprint": "cabe32d7",
    "mnemonic": "bunker shed useless about build taste comfort acquire food defense nation cement oblige race manual narrow merit lumber slight pattern plate budget armed undo",
    "name": "a1",
    "xprv": "tprv8ZgxMBicQKsPd1QusvXvAjuJN8yhsR94QMwEaiLXZ6sBaDUZfWcXqvisortPkUoAk1vdsMn6rCSv6dhRP5J4igdouEV5gcBgWNPE4ZuHfbZ",
    "xpub": "tpubD6NzVbkrYhZ4WUShmaCWa9ZQwAVe2kKxyfY1sENpyNfaQhjLHuS82RLjz19gaFTRknZhmSVAbzbeE79RjTb5coEjsjA4yg9seCLK8EFm5Q6"
  },
  "private_file": "$HOME/.firma/testnet/keys/a1/PRIVATE.json",
  "public_file": "$HOME/.firma/testnet/keys/a1/public.json",
  "public_qr_files": [
    "$HOME/.firma/testnet/keys/a1/qr/qr.png"
  ]
}
```

We could have encrypted the key before saving on disk, for example leveraging existing gpg setups like so

```
 # encryption key creation and storage in encrypted gpg
  dd if=/dev/urandom bs=1 count=32 | gpg --encrypt >encryption_key.gpg

  # bitcoin private key creation
  gpg --decrypt encryption_key.gpg | firma-offline --read-stdin random --key-name a1
```
in this latter case the key file `~/.firma/testnet/keys/a1/PRIVATE.json` looks like this
```json
{
  "t": "encrypted",
  "c": {
    "t": "base64",
    "c": "BKiWANkLynJqncOkU/d2uFY+hh8rZcaM+92xCj5H1+RBrPScxz/SAxT6hYUW1R+BiPIj1KMVSenVQWgYFoV02b6DV8uC7pKTqkFwETNavG9ZZDZCQyEB4c4EnerqAyaDLQrw5y9eec/ChFh99k7n/oPMkP0sBdEw2LNod8G69DCOmU/BT20XbnXDwgNOA94R+QWSH4zLlGsOUXjb76IqTT9SYB/tOiRGZrgUj/1VpyLc3qebVI2aLzY3r3Ent+BMkq7UjdI1SJWtu0f45OWqUhEmlsUim38pvgVYPgYfUJMpIV510Zaq4l3H5y1G67NlFLFfQo0RRDAx6s7K4Awio+Aj/raby5RjaW2kK+LhjdS8E4jKil8wdQD7zw6MSCnsea7QcLmWRe7U7I7MVTLn513y2xQRK+RXySMs0wGxngU93zdCGeNmcbywaBPl+1Z0Yv2cM4SGTPMsQxeSV20pqoNrvw1Y8Ys5zk5Gs0SMBhMTknfxlxMexkn+ZCEqAcfJHaj1SIaF/nmMLz6IdoPXKPw1FgUaZON0k0IXYpslO/5FYfNbWCNwmJXZfi+AyxLdf4lck7Hm0u87TkLuVgsChv/693qtFgS16ZuTqsU+gjzGRxIqGj47+0xO5ahGjY/HGTbJCGI="
  }
}
```

## Create second Master Key

This one is created providing dice launches:
```
firma-offline dice --key-name a2 --faces 20 -l 12 -l 11 -l 1 -l 1 -l 16 -l 8 -l 1 -l 12 -l 7 -l 4 -l 12 -l 8 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 18 -l 19 -l 12 -l 1 -l 16 -l 1 -l 18 -l 1 -l 13 -l 1 -l 1 -l 16 -l 4 -l 3 -l 1 -l 1 -l 1 -l 1 -l 1 -l 20 -l 19 -l 18 -l 17 -l 12 -l 2
```
```json
{
  "key": {
    "dice": {
      "faces": 20,
      "launches": "[12, 11, 1, 1, 16, 8, 1, 12, 7, 4, 12, 8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 18, 19, 12, 1, 16, 1, 18, 1, 13, 1, 1, 16, 4, 3, 1, 1, 1, 1, 1, 20, 19, 18, 17, 12, 2]",
      "value": "33146769803929392242686007705600000000000000300775117168313561497600063822621"
    },
    "fingerprint": "7938c502",
    "mnemonic": "enable drive anxiety pigeon bag hole invest motion notable rigid eyebrow annual meat promote embark boss pottery prison post tomorrow never plunge hockey say",
    "name": "a2",
    "xprv": "tprv8ZgxMBicQKsPdGNW7N9EPsGpWBc56L8kKncoZfxC83M5ipBV2fMhujCBnxiTx33HnqiERg6fYsgKVBdSMNKm8nEcESHfUXAUecnyWnrx6Ls",
    "xpub": "tpubD6NzVbkrYhZ4WjQJ11opoGvw5D81FfKeu6DarBzVYK9UZJSFf4BJ6Dp3y8WeYWvgA5LXAjx4T2pjYVNTxBGAjwEHrcc7Q2Smkcy8VRQX62Y"
  },
  "private_file": "$HOME/.firma/testnet/keys/a2/PRIVATE.json",
  "public_file": "$HOME/.firma/testnet/keys/a2/public.json",
  "public_qr_files": [
    "$HOME/.firma/testnet/keys/a2/qr/qr.png"
  ]
}
```

# Create the 2of2 multisig wallet

For the example we are using the two master_key created in the previous step. From the offline machines 
copy `$HOME/.firma/testnet/keys/a1/public.json` and `$HOME/.firma/testnet/keys/a2/public.json` to the 
online machine. 
`COOKIE_FILE` must point to the bitcoin node cookie file (eg. `~/.bitcoin/testnet3/.cookie`)

```
firma-online --wallet-name firma-wallet create-wallet --url http://127.0.0.1:18332 --cookie-file $COOKIE_FILE -r 2 --xpub-file $HOME/.firma/testnet/keys/a1/public.json --xpub-file $HOME/.firma/testnet/keys/a2/public.json
```

```json
{
  "qr_files": [
    "/Users/casatta/.firma/testnet/wallets/firma-wallet/qr/qr.bmp"
  ],
  "wallet": {
    "created_at_height": 1899528,
    "descriptor": "wsh(multi(2,tpubD6NzVbkrYhZ4WUShmaCWa9ZQwAVe2kKxyfY1sENpyNfaQhjLHuS82RLjz19gaFTRknZhmSVAbzbeE79RjTb5coEjsjA4yg9seCLK8EFm5Q6/0/*,tpubD6NzVbkrYhZ4WjQJ11opoGvw5D81FfKeu6DarBzVYK9UZJSFf4BJ6Dp3y8WeYWvgA5LXAjx4T2pjYVNTxBGAjwEHrcc7Q2Smkcy8VRQX62Y/0/*))#da6q39w9",
    "fingerprints": [
      "7938c502",
      "cabe32d7"
    ],
    "name": "firma-wallet",
    "required_sig": 2
  },
  "wallet_file": "/Users/casatta/.firma/testnet/wallets/firma-wallet/descriptor.json"
}
```

Note wallet file `descriptor.json` could be signed with one of the participant key using the `sign_wallet` command, this prevent an attacker to tamper with the file without getting noticed (command like `print` and `list` accept a flag to not show wallet without a signature)

## Create a receiving address

Create a new address from the just generated wallet. Bitcoin node parameters are not needed anymore since have been saved in `$HOME/.firma/testnet/firma-wallet/descriptor.json`

```
firma-online --wallet-name firma-wallet get-address
```
```json
{
  "address": "tb1qz2h8n70cnp0w6290scdl5ycvm0z7sqkrlgy5kgkds0n0fp7wwk6qyn8ywd",
  "path": "m/0/0"
}
```
State of indexes is saved in `.firma/testnet/wallets/firma-wallet/indexes.json` and by calling the command again we have:
```json
{
  "address": "tb1qmttlaqltr5kmhxuqvha9cul92c5gt9rp3zmqgu4l7pghn7z8qqascs0dfx",
  "path": "m/0/1"
}
```

Send some funds to `tb1qz2h8n70cnp0w6290scdl5ycvm0z7sqkrlgy5kgkds0n0fp7wwk6qyn8ywd`

## Check balance and coins

```
firma-online --wallet-name firma-wallet balance
```
```json
{
  "confirmed": {
    "btc": "0.00000000",
    "satoshi": 0
  },
  "pending": {
    "btc": "0.00002000",
    "satoshi": 2000
  }
}
```
```
firma-online --wallet-name firma-wallet list-coins 
```
```
{
  "coins": [
    {
      "amount": 2000,
      "outpoint": "1c9bbb8df5a03433f9cc3e77c102e06eda016ba7ae846166fa005c3db8b97ea1:0",
      "unconfirmed": true
    }
  ]
}
```

## Create the PSBT

After funds receive a confirmation we can create the PSBT specifiying the recipient and the amount, you can specify more than one recipient and you can explicitly spend specific utxo with `--coin`. See `firma-online create-tx --help`

```
firma-online --wallet-name firma-wallet create-tx --recipient tb1qnxv2x36fk6qhg3623jmsvy0x8d97jsvf0n5vyy:1400 --psbt-name test
```
```json
{
  "address_reused": [],
  "funded_psbt": {
    "name": "test",
    "psbt": "cHNidP8BAH0CAAAAAaF+ubg9XAD6ZmGErqdrAdpu4ALBdz7M+TM0oPWNu5scAAAAAAD+////AqMBAAAAAAAAIgAgaMI+YcHlEUY9mvkIVax3/a4d42jXZgcjrbWqpKNFp7l4BQAAAAAAABYAFJmYo0dJtoF0R0qMtwYR5jtL6UGJAAAAAAX8bmFtZQZ0ZXN0LWEAAQEr0AcAAAAAAAAiACASrnn5+Jhe7Sivhhv6EwzbxegCw/oJSyLNg+b0h851tAEFR1IhAuOPRdJj6043K51DVaw+MIyMHEBOuEGrv89me8fOaQLpIQLgQluouXrqa42FwP/Ki9mwFxHFQy/50SN4Zcn73HSwZFKuIgYC4EJbqLl66muNhcD/yovZsBcRxUMv+dEjeGXJ+9x0sGQMeTjFAgAAAAAAAAAAIgYC449F0mPrTjcrnUNVrD4wjIwcQE64Qau/z2Z7x85pAukMyr4y1wAAAAAAAAAAAAEBR1IhAkiG9PSdcBAS08R6LIRS6iGFbQ5ZbjY2an2EMxXcmGbPIQM2XzQNCYGxaFTBlw1c4XU4hQxj7p7ntZZDaVLjrJg39VKuIgICSIb09J1wEBLTxHoshFLqIYVtDlluNjZqfYQzFdyYZs8Myr4y1wEAAAABAAAAIgIDNl80DQmBsWhUwZcNXOF1OIUMY+6e57WWQ2lS46yYN/UMeTjFAgEAAAABAAAAAAA="
  },
  "psbt_file": "$HOME/.firma/testnet/psbts/test/psbt.json",
  "qr_files": [
    "$HOME/.firma/testnet/psbts/test/qr/qr-0.png",
    "$HOME/.firma/testnet/psbts/test/qr/qr-1.png"
  ]
}
```

## Sign from node A

```
firma-offline sign ~/.firma/testnet/psbts/test/psbt.json --key $HOME/.firma/testnet/keys/a1/PRIVATE.json --wallet-descriptor-file ~/.firma/testnet/wallets/firma-wallet/descriptor.json
```
```json
{
  "balances": "firma-wallet: -0.00001581 BTC",
  "fee": {
    "absolute": 181,
    "absolute_fmt": "0.00000181 BTC",
    "rate": 1.0168539325842696
  },
  "info": [
    "Privacy: outputs have different script types https://en.bitcoin.it/wiki/Privacy#Sending_to_a_different_script_type",
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "1c9bbb8df5a03433f9cc3e77c102e06eda016ba7ae846166fa005c3db8b97ea1:0",
      "signatures": [
        "cabe32d7"
      ],
      "value": "0.00002000 BTC",
      "wallet_with_path": "[firma-wallet]m/0/0"
    }
  ],
  "outputs": [
    {
      "address": "tb1qdrprucwpu5g5v0v6lyy9ttrhlkhpmcmg6anqwgadkk42fg6957uscl8cc6",
      "value": "0.00000419 BTC",
      "wallet_with_path": "[firma-wallet]m/1/1"
    },
    {
      "address": "tb1qnxv2x36fk6qhg3623jmsvy0x8d97jsvf0n5vyy",
      "value": "0.00001400 BTC"
    }
  ],
  "psbt_file": "$HOME/.firma/testnet/psbts/test/psbt.json",
  "size": {
    "estimated": 178,
    "psbt": 643,
    "unsigned": 125
  }
}
```

The psbt.json  at `~/.firma/testnet/psbts/test/psbt.json` now has 1 signature.

Note: if the key is encrypted, any command using the key like `sign`, need to be fed with the encryption_key
```
gpg --decrypt encryption_key.gpg | firma-offline --read-stdin sign ~/.firma/testnet/psbts/test/psbt.json --key $HOME/.firma/testnet/keys/a1/PRIVATE.json --wallet-descriptor-file ~/.firma/testnet/wallets/firma-wallet/descriptor.json
```

## Sign from node B

```
firma-offline sign ~/.firma/testnet/psbts/test/psbt.json --key $HOME/.firma/testnet/keys/r1/PRIVATE.json --wallet-descriptor-file ~/.firma/testnet/wallets/firma-wallet/descriptor.json
```
```json
{
  "balances": "firma-wallet: -0.00001581 BTC",
  "fee": {
    "absolute": 181,
    "absolute_fmt": "0.00000181 BTC",
    "rate": 1.0168539325842696
  },
  "info": [
    "Privacy: outputs have different script types https://en.bitcoin.it/wiki/Privacy#Sending_to_a_different_script_type",
    "Added paths",
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "1c9bbb8df5a03433f9cc3e77c102e06eda016ba7ae846166fa005c3db8b97ea1:0",
      "signatures": [
        "7938c502",
        "cabe32d7"
      ],
      "value": "0.00002000 BTC",
      "wallet_with_path": "[firma-wallet]m/0/0"
    }
  ],
  "outputs": [
    {
      "address": "tb1qdrprucwpu5g5v0v6lyy9ttrhlkhpmcmg6anqwgadkk42fg6957uscl8cc6",
      "value": "0.00000419 BTC",
      "wallet_with_path": "[firma-wallet]m/1/1"
    },
    {
      "address": "tb1qnxv2x36fk6qhg3623jmsvy0x8d97jsvf0n5vyy",
      "value": "0.00001400 BTC"
    }
  ],
  "psbt_file": "$HOME/.firma/testnet/psbts/test/psbt.json",
  "size": {
    "estimated": 178,
    "psbt": 751,
    "unsigned": 125
  }
}
```

## Combine, finalize and send TX

```
firma-online --wallet-name firma-wallet send-tx --psbt-file ~/.firma/testnet/psbts/test/psbt.json  --broadcast
```

```
{
  "broadcasted": true,
  "hex": "02000000000101a17eb9b83d5c00fa666184aea76b01da6ee002c1773eccf93334a0f58dbb9b1c0000000000feffffff02a30100000000000022002068c23e61c1e511463d9af90855ac77fdae1de368d7660723adb5aaa4a345a7b978050000000000001600149998a34749b68174474a8cb70611e63b4be941890400473044022035d77e5540f64b785430469965bcb015472a69461eaf29b628b3852c4d16217c022072bd4d1790ce201761f316a7ce87976fe2f470031141d0b26d61777e3b5ed9e101483045022100b735d13b126178fd929b69fe0601fa3b08111618083ed884f95d8e1eabe3561c02203a8fff2ef514a4bdb46b9d37474c9bb39f343f6e19b35e04b47faffdd294bd8f0147522102e38f45d263eb4e372b9d4355ac3e308c8c1c404eb841abbfcf667bc7ce6902e92102e0425ba8b97aea6b8d85c0ffca8bd9b01711c5432ff9d1237865c9fbdc74b06452ae00000000",
  "txid": "54233ffea203f5dd2810ed12cd811bab53b441d51a75c26cbf6fef862fe984ec"
}
```

View tx [54233ffea203f5dd2810ed12cd811bab53b441d51a75c26cbf6fef862fe984ec](https://blockstream.info/testnet/tx/54233ffea203f5dd2810ed12cd811bab53b441d51a75c26cbf6fef862fe984ec)


