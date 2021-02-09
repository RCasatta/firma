WALLET_DIR=~/.firma/testnet/wallets/signed-wallet3/
SIGNATURE_FILE=${WALLET_DIR}signature.json
WALLET_FILE=${WALLET_DIR}descriptor.json
ADDRESS=$(cat $SIGNATURE_FILE | jq -r .address)
SIGNATURE=$(cat $SIGNATURE_FILE | jq -r .signature)
WALLET=$(cat $WALLET_FILE)

bitcoin-cli -testnet -conf=/Volumes/Transcend/bitcoin-testnet/bitcoin-testnet.conf verifymessage $ADDRESS $SIGNATURE $WALLET