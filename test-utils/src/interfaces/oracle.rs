use crate::interfaces::{
    pyth_oracle::PythCore,
    redstone_oracle::RedstoneCore,
    stork_oracle::StorkCore,
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
        programs::calls::ContractDependency,
        types::{bech32::Bech32ContractId, errors::Error},
    };

    pub async fn get_price<T: Account>(
        oracle: &ContractInstance<Oracle<T>>,
        pyth: &Option<PythCore<T>>,
        stork: &Option<StorkCore<T>>,
        redstone: &Option<RedstoneCore<T>>,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        // Deploy with contract dependencies, if they are provided.
        let mut with_contracts: Vec<&dyn ContractDependency> = Vec::new();

        // Stork -> Pyth -> Redstone
        if let Some(stork) = stork {
            with_contracts.push(stork);
        }

        if let Some(pyth) = pyth {
            with_contracts.push(pyth);
        }

        if let Some(redstone) = redstone {
            with_contracts.push(redstone);
        }

        let mut with_contract_ids: Vec<Bech32ContractId> = Vec::new();

        // Stork -> Pyth -> Redstone
        if let Some(stork) = stork {
            with_contract_ids.push(stork.contract_id().into());
        }

        if let Some(pyth) = pyth {
            with_contract_ids.push(pyth.contract_id().into());
        }

        if let Some(redstone) = redstone {
            with_contract_ids.push(redstone.contract_id().into());
        }

        with_contract_ids.push(oracle.implementation_id.into());
        with_contract_ids.push(oracle.contract.contract_id().into());

        oracle
            .contract
            .methods()
            .get_price()
            .with_contracts(&with_contracts)
            .with_contract_ids(&with_contract_ids)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_debug_timestamp<T: Account>(
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

    pub async fn set_redstone_config<T: Account>(
        oracle: &ContractInstance<Oracle<T>>,
        redstone: &RedstoneCore<T>,
        config: RedstoneConfig,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .contract
            .methods()
            .set_redstone_config(config)
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
