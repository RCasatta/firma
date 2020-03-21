use bitcoin::util::amount::Denomination;
use bitcoincore_rpc::RpcApi;
use firma::*;
use serde_json::{to_value, Value};

impl crate::Wallet {
    pub fn balance(&self) -> Result<Value> {
        let balance = self.client.get_balance(Some(0), Some(true))?;
        log::info!("{}", balance);
        let satoshi = balance.as_sat();
        let btc = format!("{}", balance.to_string_in(Denomination::Bitcoin));
        let balance = BalanceOutput { satoshi, btc };
        Ok(to_value(balance)?)
    }
}
