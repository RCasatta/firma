# Android

![Screenshot_20200523-172426](https://images.casatta.it/Screenshot_20200523-172426.png?h=500)
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
  "id": {
    "kind": "MasterSecret",
    "name": "alice",
    "network": "testnet"
  },
  "key": "tprv8ZgxMBicQKsPfEf2t9eG7j14CDjS3JWL9nY3wgg6ZsLKY4tsR4wZjYuLsXWdyBPrMPo73JgeKmbd8pTkZZgQNWTdvCtDuauf52XGKL9zTDw"
}
```

```json
{
  "id": {
    "kind": "MasterSecret",
    "name": "bob",
    "network": "testnet"
  },
  "key": "tprv8ZgxMBicQKsPetwSbvkSob1PLvNeBzHftBgG61S37ywMpsCnKMkUhPbKp7FyZDsU2QvMqbF797DRqmwedPQnR5qqmUBkFVb7iNeKcEZv3ck"
}
```

### Alice & Bob Wallet

To import the following wallet descriptor, go to "select wallet" -> "+" -> "Insert manually" and paste the following json

```json
{
  "created_at_height": 1934646,
  "descriptor": "wsh(multi(2,[a2ebe04e/48'/1'/0'/2']tpubDEXDRpvW2srXCSjAvC36zYkSE3jxT1wf7JXDo35Ln4NZpmaMNhq8o9coH9U9BQ5bAN4WDGxXV9d426iYKGorFF5wvv4Wv63cZsCotiXGGkD/0/*,[1f5e43d8/48'/1'/0'/2']tpubDFU4parcXvV8tBYt4rS4a8rGNF1DA32DCnRfhzVL6b3MSiDomV95rv9mb7W7jAPMTohyEYpbhVS8FbmTsuQsFRxDWPJX2ZFEeRPMFz3R1gh/0/*))#szg2xsau",
  "id": {
    "kind": "Wallet",
    "name": "alice-and-bob",
    "network": "testnet"
  },
  "required_sig": 2
}
```

### Alice & Bob transaction to Carol

To import the following transaction (PSBT) copy the text, go to "select transaction" -> "+" -> "From Clipboard"

NOTE: take care not to copy trailing spaces

```
cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAF/G5hbWUIdG8tY2Fyb2wAAQChAgAAAAEbuYvreUkM84tDJuxdjxZmErxAyO/PkP+ozooG1kBiZAAAAAAjIgAg/KddPamHVwK3NnYT58PR3q+a5k9zwFC8zJXE6Nwr5zX9////AkyLBgAAAAAAF6kUZ3Eos+P2CT0g41zAxb+TPZLthgiHpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qn6FHQABASukzgAAAAAAACIAIPWRVyIca9Av7u6ho2/suW3Z5lT52N7R+fN7dkvB8j6qAQVHUiECkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4hA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gUq4iBgKSs5xbbfPnypRjnI4Cs8JhIMvDtnre2rZTX3QOW2/NLhyi6+BOMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDfXRZ174HHQOH5i1m5cCiW7s2s7oEt3c3Oj65Q6tn3eAcH15D2DAAAIABAACAAAAAgAIAAIAAAAAAAAAAAAAAAQFHUiEC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAhA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/Uq4iAgLjgp6MBzab6rhhE/ElAh6pu5UspqkrJbtlOAdlXWfYQByi6+BOMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDqOISXqlaOQ55ST8/9mFJ96Hci1iByNRgThG83aJCUX8cH15D2DAAAIABAACAAAAAgAIAAIAAAAAAAQAAAAA=
```
