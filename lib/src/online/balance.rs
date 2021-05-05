use crate::online::WalletNameOptions;
use crate::*;
use bitcoin::util::amount::Denomination;
use bitcoin::Amount;
use bitcoincore_rpc::RpcApi;

impl OnlineContext {
    pub fn balance(&self, opt: &WalletNameOptions) -> Result<BalanceOutput> {
        let client = self.make_client(&opt.wallet_name)?;
        let balances: Balances = client.call("getbalances", &[])?;
        let am = balances.watchonly.immature + balances.watchonly.untrusted_pending;
        let pending = match am.as_sat() {
            0 => None,
            _ => Some(am.into()),
        };
        let confirmed: BalanceSatBtc = balances.watchonly.trusted.into();
        let balance = BalanceOutput { confirmed, pending };
        Ok(balance)
    }
}

impl From<Amount> for BalanceSatBtc {
    fn from(a: Amount) -> Self {
        let satoshi = a.as_sat();
        let btc = a.to_string_in(Denomination::Bitcoin);
        BalanceSatBtc { satoshi, btc }
    }
}
