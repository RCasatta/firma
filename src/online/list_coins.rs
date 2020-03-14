use bitcoincore_rpc::RpcApi;

impl crate::Wallet {
    pub fn list_coins(&self) -> firma::Result<()> {
        let mut list_coins = self.client.list_unspent(None, None, None, None, None)?;
        list_coins.sort_by(|a, b| a.amount.cmp(&b.amount));
        for utxo in list_coins.iter() {
            log::info!("{}:{} {}", utxo.txid, utxo.vout, utxo.amount);
        }

        Ok(())
    }
}
