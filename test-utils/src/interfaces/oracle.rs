use crate::interfaces::{
    pyth_oracle::PythCore, redstone_oracle::RedstoneCore, stork_oracle::StorkCore,
};
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/oracle-contract/out/debug/oracle-contract-abi.json"
));

// 10 minutes
pub const ORACLE_TIMEOUT: u64 = 600;

pub mod oracle_abi {

    use crate::data_structures::ContractInstance;

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        programs::calls::Execution,
        types::errors::Error,
    };

    pub async fn initialize<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
        stork: Option<StorkConfig>,
        pyth: Option<PythConfig>,
        redstone: Option<RedstoneConfig>,
    ) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .initialize(stork, pyth, redstone)
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }

    pub async fn get_last_good_price<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
    ) -> Result<CallResponse<Price>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .get_last_good_price()
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn get_price<T: Account + Clone>(oracle: &ContractInstance<Oracle<T>>) -> u64 {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .get_price()
            .with_tx_policies(tx_params)
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await
            .unwrap()
            .value
    }

    pub async fn get_stork_price<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
    ) -> Option<(u64, u64)> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .get_stork_price()
            .determine_missing_contracts()
            .await
            .unwrap()
            .with_tx_policies(tx_params)
            .simulate(Execution::state_read_only())
            .await
            .unwrap()
            .value
    }

    pub async fn set_debug_timestamp<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
        timestamp: u64,
    ) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .set_debug_timestamp(timestamp)
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }

    pub async fn set_stork_config<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
        stork: &StorkCore<T>,
        config: StorkConfig,
    ) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .set_stork_config(Some(config))
            .with_contracts(&[stork])
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }

    pub async fn set_pyth_config<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
        pyth: &PythCore<T>,
        config: PythConfig,
    ) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .set_pyth_config(Some(config))
            .with_contracts(&[pyth])
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }

    pub async fn set_redstone_config<T: Account + Clone>(
        oracle: &ContractInstance<Oracle<T>>,
        redstone: &RedstoneCore<T>,
        config: RedstoneConfig,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .set_redstone_config(Some(config))
            .with_contracts(&[redstone])
            .with_contract_ids(&[
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
    }
}
