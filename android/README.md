# Android

![Screenshot_20200523-172426](https://images.casatta.it/Screenshot_20200523-172426.png?h=500)
![Screenshot_20200523-172608](https://images.casatta.it/Screenshot_20200523-172608.png?h=500)
![Screenshot_20200523-172628](https://images.casatta.it/Screenshot_20200523-172628.png?h=500)

## Beta test

Checkout releases at https://github.com/RCasatta/firma/releases 

Get testnet version from google play https://play.google.com/store/apps/details?id=it.casatta.testnet&hl=en

## Building

Check the steps made in the [CI workflow](https://github.com/RCasatta/firma/blob/master/.github/workflows/rust.yml) in the `android-lib` and `android-apk` jobs

## Example
### Alice & Bob Keys

The following 2 keys are from Alice and Bob, they will use their keys for the 2of2 wallet in the next section

```json
{
  "mnemonic": "quiz knock second dose baby funny need peanut cycle own sponsor walk major rose alter axis visa clip target age chair visit trouble today",
  "xpub": "tpubD6NzVbkrYhZ4YhgpmoJrX8fAmFFNCdhEj68qECiPz98iNZ9e3Tm9v3XD3fzHZfBoLqeSm9oLtighoeijQ9jGAFm9raQ4JqHZ1N4BHyaBz6Y",
  "xprv": "tprv8ZgxMBicQKsPfEf2t9eG7j14CDjS3JWL9nY3wgg6ZsLKY4tsR4wZjYuLsXWdyBPrMPo73JgeKmbd8pTkZZgQNWTdvCtDuauf52XGKL9zTDw",
  "name": "alice",
  "fingerprint": "a2ebe04e"
}
```

```json
{
  "mnemonic": "proof senior abstract clock mercy penalty pet library ramp heavy high primary meadow fish own mother gym civil awesome item walnut outdoor woman tennis",
  "xpub": "tpubD6NzVbkrYhZ4YMyEVaR3CzfVuwtaMKUaTVH3NXULYFjkfMTYwka4stDBzHhHkxd4MEMVgyyEV1WBCrpwde72w8LzjAE6oRLARBAiCD8cGQV",
  "xprv": "tprv8ZgxMBicQKsPetwSbvkSob1PLvNeBzHftBgG61S37ywMpsCnKMkUhPbKp7FyZDsU2QvMqbF797DRqmwedPQnR5qqmUBkFVb7iNeKcEZv3ck",
  "name": "bob",
  "fingerprint": "1f5e43d8"
}
```

### Alice & Bob Wallet

To import the following wallet descriptor copy the json, go to "select wallet" -> "+" -> "From Clipboard"

```json
{
  "name": "alice-and-bob",
  "descriptor_main": "wsh(multi(2,tpubD6NzVbkrYhZ4YhgpmoJrX8fAmFFNCdhEj68qECiPz98iNZ9e3Tm9v3XD3fzHZfBoLqeSm9oLtighoeijQ9jGAFm9raQ4JqHZ1N4BHyaBz6Y/0/*,tpubD6NzVbkrYhZ4YMyEVaR3CzfVuwtaMKUaTVH3NXULYFjkfMTYwka4stDBzHhHkxd4MEMVgyyEV1WBCrpwde72w8LzjAE6oRLARBAiCD8cGQV/0/*))#wss3kl0z",
  "descriptor_change": "wsh(multi(2,tpubD6NzVbkrYhZ4YhgpmoJrX8fAmFFNCdhEj68qECiPz98iNZ9e3Tm9v3XD3fzHZfBoLqeSm9oLtighoeijQ9jGAFm9raQ4JqHZ1N4BHyaBz6Y/1/*,tpubD6NzVbkrYhZ4YMyEVaR3CzfVuwtaMKUaTVH3NXULYFjkfMTYwka4stDBzHhHkxd4MEMVgyyEV1WBCrpwde72w8LzjAE6oRLARBAiCD8cGQV/1/*))#dsqynmm3",
  "fingerprints": [
    "1f5e43d8",
    "a2ebe04e"
  ],
  "required_sig": 2,
  "created_at_height": 1835680
}
```

### Alice & Bob transaction to Carol

To import the following transaction (PSBT) copy the text, go to "select transaction" -> "+" -> "From Clipboard"

NOTE: take care not to copy trailing spaces

```
cHNidP8BAFMCAAAAASFSbAAqstjwTxbGtWir21+meBp5LMcUQsBSgZ5bDtD7AQAAAAD+////AV6rCAAAAAAAF6kU4wEfjwloN3dvCV9wNOekdO53E92HAAAAAAX8bmFtZQh0by1jYXJvbAABAKECAAAAAcyd+J9zW1wSNV/mozPMv8mcXFzwQrK1EKq/FvRPJS40AQAAACMiACC+U25ZjJg9CiGsPhlAqQ0GWtFhOWxqopXdDTrh2oBdEP3///8Cp0lVAAAAAAAXqRRUIuqRoByuLh5D6zdViHWG7aGi84cVrAgAAAAAACIAIDz80EGjAUinXjMddGAtfQ3fKqcjgWj9wY5Y+8c7NA1zoAIcAAEBKxWsCAAAAAAAIgAgPPzQQaMBSKdeMx10YC19Dd8qpyOBaP3Bjlj7xzs0DXMBBUdSIQNP26ruccaqcu2cxRFYsPON2gj4ALrAFQ5ApBVtM+z9SiECIwjICs3MMHNnGbXPgSQKezAcOC5HzejKyjATzR8qXiRSriIGAiMIyArNzDBzZxm1z4EkCnswHDguR83oysowE80fKl4kDB9eQ9gAAAAAAAAAACIGA0/bqu5xxqpy7ZzFEViw843aCPgAusAVDkCkFW0z7P1KDKLr4E4AAAAAAAAAAAAA
```
