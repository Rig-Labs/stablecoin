use crate::interfaces::{pyth_oracle::PythCore, redstone_oracle::RedstoneCore};
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/oracle-contract/out/debug/oracle-contract-abi.json"
));

// 4 hours
pub const ORACLE_TIMEOUT: u64 = 14400;

pub mod oracle_abi {

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        programs::calls::ContractDependency,
        types::errors::Error,
    };

    pub async fn get_price<T: Account>(
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &Option<RedstoneCore<T>>,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        let mut with_contracts: Vec<&dyn ContractDependency> = Vec::new();
        with_contracts.push(pyth);
        if let Some(redstone) = redstone {
            with_contracts.push(redstone);
        }

        oracle
            .methods()
            .get_price()
            .with_contracts(&with_contracts)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_debug_timestamp<T: Account>(oracle: &Oracle<T>, timestamp: u64) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .set_debug_timestamp(timestamp)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }

    pub async fn set_redstone_config<T: Account>(
        oracle: &Oracle<T>,
        redstone: &RedstoneCore<T>,
        config: RedstoneConfig,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .set_redstone_config(config)
            .with_contracts(&[redstone])
            .with_tx_policies(tx_params)
            .call()
            .await
    }
}
