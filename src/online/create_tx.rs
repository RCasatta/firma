use crate::*;
use bitcoin::{Address, Amount, OutPoint};
use bitcoincore_rpc::RpcApi;
use log::{debug, info};
use serde_json::{to_value, Value};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CreateTxOptions {
    /// Address and amount in satoshi of the recipient specified as address:amount,
    /// it is possible to use units for amount but is mandatory to enclose quotes eg "address:amount BTC"
    /// at least 1 is required
    #[structopt(long, long = "recipient")]
    pub recipients: Vec<AddressAmount>,

    /// Coin to spend, specified as txid:vout see list-coins, if not specified the node will choose coins
    #[structopt(long, long = "coin")]
    pub coins: Vec<OutPoint>,
}

#[derive(StructOpt, Debug)]
pub struct AddressAmount {
    pub address: Address,
    pub amount: Amount,
}

impl CreateTxOptions {
    fn validate(&self) -> Result<()> {
        if self.recipients.is_empty() {
            return err("At least one recipient is mandatory (--recipient)");
        }

        Ok(())
    }

    fn recipients_as_outputs(&self) -> HashMap<String, Amount> {
        let mut outputs = HashMap::new();
        for recipient in self.recipients.iter() {
            outputs.insert(recipient.address.to_string(), recipient.amount.clone());
        }
        outputs
    }

    fn coins_as_inputs(&self) -> Vec<CreateRawTransactionInput> {
        let mut vec = vec![];
        for coin in self.coins.iter() {
            vec.push(CreateRawTransactionInput {
                txid: coin.txid,
                vout: coin.vout,
                sequence: None,
            });
        }
        vec
    }
}

impl FromStr for AddressAmount {
    type Err = std::io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() != 2 {
            Err(io_err("recipient is not in the format address:amount"))
        } else {
            let amount = match parts[1].parse::<u64>() {
                Ok(unsigned) => Amount::from_sat(unsigned),
                Err(_) => Amount::from_str(parts[1])
                    .map_err(|_| io_err("Amount in recipient is invalid, should be satoshi"))?,
            };
            let address = Address::from_str(parts[0])
                .map_err(|_| io_err("Address in recipient is not valid"))?;
            Ok(AddressAmount { address, amount })
        }
    }
}

impl Wallet {
    pub fn create_tx(&self, opt: &CreateTxOptions) -> Result<Value> {
        opt.validate()?;
        let outputs = opt.recipients_as_outputs();
        debug!("{:?}", outputs);
        let inputs = opt.coins_as_inputs();
        debug!("{:?}", inputs);

        let mut options: WalletCreateFundedPsbtOptions = Default::default();
        options.include_watching = Some(true);
        options.change_address = Some(self.get_address(None, true)?.address);
        let result = self.client.wallet_create_funded_psbt(
            &inputs,
            &outputs,
            None,
            Some(options),
            Some(true),
        );
        info!("wallet_create_funded_psbt {:#?}", result);

        // decreasing auto-incremented change index if error or change not used
        let funded_psbt = match result {
            Ok(value) => {
                if value.change_position == -1 {
                    self.context.decrease_change_index()?;
                }
                value
            }
            Err(e) => {
                self.context.decrease_change_index()?;
                return err(&format!("error creating psbt ({:?})", e));
            }
        };

        let psbt_file = save_psbt(&funded_psbt, &self.context.firma_datadir)?;

        let transactions = self
            .client
            .list_transactions(None, Some(1000), None, Some(true))
            .unwrap();
        let mut address_reused = HashSet::new();
        for recipient in opt.recipients.iter() {
            for tx in transactions.iter() {
                if tx.detail.address == recipient.address
                    && tx.detail.category == GetTransactionResultDetailCategory::Send
                {
                    address_reused.insert(recipient.address.clone());
                }
            }
        }

        let create_tx = CreateTxOutput {
            funded_psbt,
            psbt_file,
            address_reused,
        };

        Ok(to_value(create_tx)?)
    }
}
