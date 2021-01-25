use crate::online::WalletNameOptions;
use crate::*;
use bitcoin::OutPoint;
use bitcoincore_rpc::RpcApi;

impl Context {
    pub fn list_coins(&self, opt: &WalletNameOptions) -> Result<ListCoinsOutput> {
        let client = self.make_client(&opt.wallet_name)?;
        let mut list_coins = client.list_unspent(Some(0), None, None, None, None)?;
        list_coins.sort_by(|a, b| a.amount.cmp(&b.amount));
        let mut coins = vec![];
        for utxo in list_coins.iter() {
            log::info!("{}:{} {}", utxo.txid, utxo.vout, utxo.amount);
            let outpoint = OutPoint::new(utxo.txid, utxo.vout);
            let amount = utxo.amount.as_sat();
            let unconfirmed = if utxo.confirmations == 0 {
                Some(true)
            } else {
                None
            };
            coins.push(Coin {
                outpoint,
                amount,
                unconfirmed,
            });
        }
        coins.sort_by(|a, b| a.amount.cmp(&b.amount));
        let list_coins = ListCoinsOutput { coins };

        Ok(list_coins)
    }
}
