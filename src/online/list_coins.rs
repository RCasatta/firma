use bitcoin::OutPoint;
use bitcoincore_rpc::RpcApi;
use firma::*;

impl crate::Wallet {
    pub fn list_coins(&self) -> Result<ListCoinsOutput> {
        let mut list_coins = self.client.list_unspent(None, None, None, None, None)?;
        list_coins.sort_by(|a, b| a.amount.cmp(&b.amount));
        let mut coins = vec![];
        for utxo in list_coins.iter() {
            log::info!("{}:{} {}", utxo.txid, utxo.vout, utxo.amount);
            let outpoint = OutPoint::new(utxo.txid, utxo.vout);
            let amount = utxo.amount.as_sat();
            coins.push(Coin { outpoint, amount });
        }
        let list_coins = ListCoinsOutput { coins };

        Ok(list_coins)
    }
}
