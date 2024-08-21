use crate::interfaces::{pyth_oracle::PythCore, redstone_oracle::RedstoneCore};
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/mock-oracle-contract/out/debug/mock-oracle-contract-abi.json"
));

// 1 minute
pub const ORACLE_TIMEOUT: u64 = 60;

pub mod oracle_abi {

    use super::*;
    use fuels::prelude::{Account, TxPolicies};

    pub async fn get_price<T: Account>(
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);
        oracle
            .methods()
            .get_price()
            .with_contracts(&[pyth, redstone])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }
}
