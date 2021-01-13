use crate::offline::sign::save_psbt;
use crate::online::get_address::GetAddressOptions;
use crate::online::Wallet;
use crate::qr::QrMode;
use crate::*;
use bitcoin::{Address, Amount, OutPoint};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    CreateRawTransactionInput, GetTransactionResultDetailCategory, WalletCreateFundedPsbtOptions,
};
use bitcoincore_rpc::RpcApi;
use log::{debug, info};
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

    /// Name of the PSBT
    #[structopt(short, long)]
    pub psbt_name: String,

    /// QR code max version to use (max size)
    #[structopt(long, default_value = "14")]
    pub qr_version: i16,
}

#[derive(StructOpt, Debug)]
pub struct AddressAmount {
    pub address: Address,
    pub amount: Amount,
}

impl CreateTxOptions {
    fn validate(&self) -> Result<()> {
        if self.recipients.is_empty() {
            return Err("At least one recipient is mandatory (--recipient)".into());
        }

        Ok(())
    }

    fn recipients_as_outputs(&self) -> HashMap<String, Amount> {
        self.recipients
            .iter()
            .map(|r| (r.address.to_string(), r.amount))
            .collect()
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
    pub fn create_tx(&self, opt: &CreateTxOptions) -> Result<CreateTxOutput> {
        opt.validate()?;
        let outputs = opt.recipients_as_outputs();
        debug!("{:?}", outputs);
        let inputs = opt.coins_as_inputs();
        debug!("{:?}", inputs);

        let get_addr_opts = GetAddressOptions {
            index: None,
            qr_mode: QrMode::None,
        };

        let options = WalletCreateFundedPsbtOptions {
            include_watching: Some(true),
            change_address: Some(self.get_address(&get_addr_opts)?.address),
            ..Default::default()
        };

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
                    self.context.decrease_index()?;
                }
                value
            }
            Err(e) => {
                self.context.decrease_index()?;
                return Err(format!("error creating psbt ({:?})", e).into());
            }
        };

        let mut psbt = psbt_from_rpc(&funded_psbt, &opt.psbt_name)?;

        let mut psbts_dir = self.context.psbts_dir()?;
        let (psbt_file, qr_files) = save_psbt(&mut psbt, &mut psbts_dir, opt.qr_version)?;

        // detect address reuse
        let transactions = self
            .client
            .list_transactions(None, Some(1000), None, Some(true))
            .unwrap();
        let mut address_reused = HashSet::new();
        for recipient in opt.recipients.iter() {
            for tx in transactions.iter() {
                if tx.detail.address.as_ref() == Some(&recipient.address)
                    && tx.detail.category == GetTransactionResultDetailCategory::Send
                {
                    address_reused.insert(recipient.address.clone());
                }
            }
        }

        let create_tx = CreateTxOutput {
            funded_psbt: (&psbt).into(),
            psbt_file,
            address_reused,
            qr_files,
        };

        Ok(create_tx)
    }
}
