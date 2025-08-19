use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "DefaultPool",
    abi = "contracts/default-pool-contract/out/debug/default-pool-contract-abi.json"
));

pub mod default_pool_abi {
    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::token::Token;
    use fuels::prelude::Account;
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, Error, TxPolicies},
        types::Identity,
    };

    pub async fn initialize<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        protocol_manager: Identity,
        active_pool: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = default_pool
            .contract
            .methods()
            .initialize(protocol_manager.clone(), active_pool)
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_usdm_debt<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        asset_id: AssetId,
    ) -> CallResponse<u64> {
        default_pool
            .contract
            .methods()
            .get_usdm_debt(asset_id.into())
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_asset<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        asset_id: AssetId,
    ) -> CallResponse<u64> {
        default_pool
            .contract
            .methods()
            .get_asset(asset_id.into())
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_usdm_debt<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        default_pool
            .contract
            .methods()
            .increase_usdm_debt(amount, asset_id.into())
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        asset_id: AssetId,
        trove_manager: Identity,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        default_pool
            .contract
            .methods()
            .add_asset(asset_id.into(), trove_manager)
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdm_debt<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        default_pool
            .contract
            .methods()
            .decrease_usdm_debt(amount, asset_id.into())
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        token: &Token<T>,
        amount: u64,
    ) -> CallResponse<()> {
        let mock_asset_id = AssetId::from(*token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(mock_asset_id);

        let tx_params = TxPolicies::default().with_tip(1);

        default_pool
            .contract
            .methods()
            .recieve()
            .with_tx_policies(tx_params)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call_params(call_params)
            .unwrap()
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
            ])
            .with_contracts(&[token])
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_active_pool<T: Account>(
        default_pool: &ContractInstance<DefaultPool<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        default_pool
            .contract
            .methods()
            .send_asset_to_active_pool(amount, asset_id.into())
            .with_contracts(&[&active_pool.contract])
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
            .unwrap()
    }
}
