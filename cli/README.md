[![MIT license](https://img.shields.io/github/license/RCasatta/firma)](https://github.com/RCasatta/firma/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/firma-cli.svg)](https://crates.io/crates/firma-cli)

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
  "id": {
    "kind": "MasterSecret",
    "name": "a1",
    "network": "testnet"
  },
  "key": "tprv8ZgxMBicQKsPdQsGb1U22Lw7bPwhbxRkV8Q1mf8mv42q6HpJS7MW5hx1J44gKK6m2pQyC32mG1i6v1P9C97MDx7MvKZzgoXTpcwUgTSEobm"
}
```

We could have encrypted the key before saving on disk, for example leveraging existing gpg setups like so

```
 # encryption key creation and storage in encrypted gpg
  dd if=/dev/urandom bs=1 count=32 | gpg --encrypt -r 'DEADBEEF!' >encryption_key.gpg

  # bitcoin private key creation
  gpg --decrypt encryption_key.gpg | firma-offline --encrypt random --key-name a1
```
in this latter case the key file `~/.firma/testnet/keys/a1/master_secret.json` looks like this
```json
{
  "t": "encrypted",
  "c": {
    "t": "base64",
    "c": "5Vqdw61WCoxyvl6mj6WBGEPzzI/SCLxwukHbbxCYsQphkPdoGDMaPhLL7Jg7Ok4yJa7E79LiiOSaRcszjnyLH3lfskF3ii2u5qTcacQhmuh5HV8d275hGAoYejY24MU58h/4Mo5A3om6woRpIgABmAEFXGCeTsjgvvXO+iD0EJA2tse+YQJRhMQGYCMeNH7BcrnWrAhhu3eBCsZdt0j5bsrP6aX3DLQIW6uUhsP7nklscGdRstu82+NEkdwonP+hBXBrkFxfe9DCer/x4IbeZF6TGA=="
  }
}
```

In this latter case to read the plain text of the key
```
gpg --decrypt encryption_key.gpg | firma-offline --encrypt export --kind MasterSecret --name a1
```
```json
{
  "id": {
    "kind": "MasterSecret",
    "name": "a1e",
    "network": "testnet"
  },
  "key": "tprv8ZgxMBicQKsPdHXHYrBsowgZXAh1bisk2nxvJLqRakJ9tZDLTLgFSGZDH79bKF19cTkmW8LHV3ZFbRytQAxjXx1MUFrrzpdfxiFcqqfpjkf"
}
```

Note: if the wallet files are encrypted, any command need to be fed with the encryption_key to encrypt and decrypt the data.

## Create second Master Key

This one is created providing dice launches:
```
firma-offline dice --key-name a2 --faces 20 -l 12 -l 11 -l 1 -l 1 -l 16 -l 8 -l 1 -l 12 -l 7 -l 4 -l 12 -l 8 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 1 -l 18 -l 19 -l 12 -l 1 -l 16 -l 1 -l 18 -l 1 -l 13 -l 1 -l 1 -l 16 -l 4 -l 3 -l 1 -l 1 -l 1 -l 1 -l 1 -l 20 -l 19 -l 18 -l 17 -l 12 -l 2
```
```json
{
  "dice": {
    "faces": 20,
    "launches": "[12, 11, 1, 1, 16, 8, 1, 12, 7, 4, 12, 8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 18, 19, 12, 1, 16, 1, 18, 1, 13, 1, 1, 16, 4, 3, 1, 1, 1, 1, 1, 20, 19, 18, 17, 12, 2]",
    "value": "33146769803929392242686007705600000000000000300775117168313561497600063822621"
  },
  "id": {
    "kind": "MasterSecret",
    "name": "a2",
    "network": "testnet"
  },
  "key": "tprv8ZgxMBicQKsPdGNW7N9EPsGpWBc56L8kKncoZfxC83M5ipBV2fMhujCBnxiTx33HnqiERg6fYsgKVBdSMNKm8nEcESHfUXAUecnyWnrx6Ls"
}
```

# Create the 2of2 multisig wallet

The first time using the `firma-online` tool you need to initiate the connection parameters:
```
firma-online connect --url http://127.0.0.1:8332 --cookie-file $HOME/.bitcoin/.cookie
```
This must be done once per network, and every time node configuration change.

For the example we are using the two master_key created in the previous step. From the offline machines 
copy `$HOME/.firma/testnet/keys/a1/descriptor_public_key.json` and `$HOME/.firma/testnet/keys/a2/descriptor_public_key.json` to the 
online machine. You may choose to copy the files to the correct destination directory, or use the `import` command which would allow to optionally encrypt this data.

```
firma-online create-wallet --wallet-name firma-wallet -r 2 --key-name a1 --key-name a2
```

```json
{
  "created_at_height": 1934912,
  "descriptor": "wsh(multi(2,[2f15d226/48'/1'/0'/2']tpubDFHFgFr6HbP88U7grBQ44yvocSU1EGkXX1dArKRum1qvb4Y6hy4CpJuPqpKZSyVnHptf6zoaW4HUjFHXgmtfy2vTGF1fccPy2ioNvKeZUnq/0/*,[7938c502/48'/1'/0'/2']tpubDEJZZGYXbZKMNEWgCdG9XZDycYM19Y8WxNc2cYcxLYhnNvKpbFNkgAza1x4GCUAHLdxx28R6dX88VhjgmZscW8Dzw6pGDzJ8a4gUCqHh1ny/0/*))#jpa3vwyx",
  "id": {
    "kind": "Wallet",
    "name": "firma-wallet",
    "network": "testnet"
  }
}
```

Note wallet file `wallet.json` could be signed with one of the participant key using the `sign_wallet` command, this prevent an attacker to tamper with the watch-only wallet without getting noticed.

## Create a receiving address

Create a new address from the just generated wallet. Bitcoin node parameters are not needed anymore since have been saved in `$HOME/.firma/testnet/firma-wallet/descriptor.json`

```
firma-online get-address --wallet-name firma-wallet 
```
```json
{
  "address": "tb1qdkl3aufvvk2zst22dy3ffjt0kfdl79mhvu6jcwecm5exm6j8dveseklast",
  "path": "m/0/0"
}
```
State of indexes is saved in `$HOME/.firma/testnet/wallets/firma-wallet/indexes.json` and by calling the command again we have:
```json
{
  "address": "tb1q8m2456wjxu8mlkf708d2yvtmtlg59awvd2l3jjzkmt37gtzmx6psva9fnl",
  "path": "m/0/1"
}
```

Send some funds to `tb1qdkl3aufvvk2zst22dy3ffjt0kfdl79mhvu6jcwecm5exm6j8dveseklast`

## Check balance and coins

```
firma-online balance --wallet-name firma-wallet
```
```json
{
  "confirmed": {
    "btc": "0.00000000",
    "satoshi": 0
  },
  "pending": {
    "btc": "0.00586300",
    "satoshi": 586300
  }
}
```
```
firma-online list-coins --wallet-name firma-wallet 
```
```
{
  "coins": [
    {
      "amount": 586300,
      "outpoint": "43d3a56e8afe96eeb1c3a260bae735d064e5946190d9fb90524047bd21dbf383:0",
      "unconfirmed": true
    }
  ]
}
```

## Create the PSBT

After funds receive a confirmation we can create the PSBT specifiying the recipient and the amount, you can specify more than one recipient and you can explicitly spend specific utxo with `--coin`. See `firma-online create-tx --help`

```
firma-online create-tx --wallet-name firma-wallet --recipient tb1q8m2456wjxu8mlkf708d2yvtmtlg59awvd2l3jjzkmt37gtzmx6psva9fnl:22400 --psbt-name test
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
firma-offline sign --psbt-name test --key-name a1 --wallet-name firma-wallet
```
```json
{
  "balances": "",
  "fee": {
    "absolute": 193,
    "absolute_fmt": "0.00000193 BTC",
    "rate": 1.0157894736842106
  },
  "info": [
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "43d3a56e8afe96eeb1c3a260bae735d064e5946190d9fb90524047bd21dbf383:0",
      "signatures": [
        "2f15d226"
      ],
      "value": "0.00586300 BTC"
    }
  ],
  "outputs": [
    {
      "address": "tb1q8m2456wjxu8mlkf708d2yvtmtlg59awvd2l3jjzkmt37gtzmx6psva9fnl",
      "value": "0.00022400 BTC"
    },
    {
      "address": "tb1qye8gt2pjwpdn8mj7eh3jgnl0hnfwq23lq4cat6s55hsnka3v2kss4jxsmm",
      "value": "0.00563707 BTC"
    }
  ],
  "psbt_file": "",
  "size": {
    "estimated": 190,
    "psbt": 1083,
    "unsigned": 137
  }
}
```

The psbt.json at `~/.firma/testnet/psbts/test/psbt.json` now has 1 signature (notice the `Added signatures` in the output).

## Sign from node B

```
firma-offline sign --psbt-name test --key-name a2 --wallet-name firma-wallet
```
```json
{
  "balances": "TODO",
  "fee": {
    "absolute": 193,
    "absolute_fmt": "0.00000193 BTC",
    "rate": 1.0157894736842106
  },
  "info": [
    "Added signatures"
  ],
  "inputs": [
    {
      "outpoint": "43d3a56e8afe96eeb1c3a260bae735d064e5946190d9fb90524047bd21dbf383:0",
      "signatures": [
        "2f15d226",
        "7938c502"
      ],
      "value": "0.00586300 BTC"
    }
  ],
  "outputs": [
    {
      "address": "tb1q8m2456wjxu8mlkf708d2yvtmtlg59awvd2l3jjzkmt37gtzmx6psva9fnl",
      "value": "0.00022400 BTC"
    },
    {
      "address": "tb1qye8gt2pjwpdn8mj7eh3jgnl0hnfwq23lq4cat6s55hsnka3v2kss4jxsmm",
      "value": "0.00563707 BTC"
    }
  ],
  "psbt_file": "",
  "size": {
    "estimated": 190,
    "psbt": 1191,
    "unsigned": 137
  }
}
```

## Combine, finalize and send TX

```
firma-online send-tx --wallet-name firma-wallet --psbt-name test --broadcast
```

```
{
  "broadcasted": true,
  "hex": "0200000000010183f3db21bd47405290fbd9906194e564d035e7ba60a2c3b1ee96fe8a6ea5d3430000000000feffffff0280570000000000002200203ed55a69d2370fbfd93e79daa2317b5fd142f5cc6abf194856dae3e42c5b3683fb99080000000000220020264e85a832705b33ee5ecde3244fefbcd2e02a3f0571d5ea14a5e13b762c55a10400473044022041785595cc34a022686213b8d7b34bac2f2bfc77e37a8fa2f2f3019b0646bbfb022047e12715e32b081be49865ffdbca73829b0231e44f77c9afcd6aa25582186e300148304502210081f298aadf2e5f68e322c030b1e97e23f45a1ac9ddf5fa4b964c1a57847f37b70220129c3d54e9f5c946511bfd30fc755846014654d4d693455fa08279a994617e410147522102e43ee99d46f46cd17d25987701576ae07129ab268ca9879e53a345546537dc862102ccbef362214e9e7ece2bcd33731cdabd0c2937d9bec9db2684fb68021297f8b752ae00000000",
  "txid": "4e08b321a79465cdbba8ad811ddaa68ffe79604406413b25b55c76b9850902e5"
}
```

View tx [4e08b321a79465cdbba8ad811ddaa68ffe79604406413b25b55c76b9850902e5](https://blockstream.info/testnet/tx/4e08b321a79465cdbba8ad811ddaa68ffe79604406413b25b55c76b9850902e5)


