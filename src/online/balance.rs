use bitcoincore_rpc::RpcApi;

impl crate::Wallet {
    pub fn balance(&self) -> firma::Result<()> {
        let balance = self.client.get_balance(Some(0), Some(true))?;
        log::info!("{}", balance);
        Ok(())
    }
}
