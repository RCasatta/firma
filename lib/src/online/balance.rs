use crate::*;
use bitcoin::util::amount::Denomination;
use bitcoincore_rpc::RpcApi;

impl Wallet {
    pub fn balance(&self) -> Result<BalanceOutput> {
        let balance = self.client.get_balance(Some(0), Some(true))?;
        log::info!("{}", balance);
        let satoshi = balance.as_sat();
        let btc = balance.to_string_in(Denomination::Bitcoin);
        let balance = BalanceOutput { satoshi, btc };
        Ok(balance)
    }
}
