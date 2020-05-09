use crate::*;
use bitcoin::OutPoint;
use bitcoincore_rpc::RpcApi;

impl Wallet {
    pub fn list_coins(&self) -> Result<ListCoinsOutput> {
        let mut list_coins = self.client.list_unspent(Some(0), None, None, None, None)?;
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
