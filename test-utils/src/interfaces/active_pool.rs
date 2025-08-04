use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "ActivePool",
    abi = "contracts/active-pool-contract/out/debug/active-pool-contract-abi.json"
));

pub mod active_pool_abi {
    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::default_pool::DefaultPool;
    use crate::interfaces::token::Token;
    use fuels::prelude::Account;
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::{
        prelude::{CallParameters, ContractId, Error, TxPolicies},
        types::{AssetId, Identity},
    };

    pub async fn initialize<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        borrow_operations: Identity,
        stability_pool: Identity,
        default_pool: ContractId,
        protocol_manager: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = active_pool
            .contract
            .methods()
            .initialize(
                borrow_operations.clone(),
                stability_pool.clone(),
                default_pool,
                protocol_manager,
            )
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_usdm_debt<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        asset_id: AssetId,
    ) -> CallResponse<u64> {
        active_pool
            .contract
            .methods()
            .get_usdm_debt(asset_id.into())
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_asset<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        asset_id: AssetId,
    ) -> CallResponse<u64> {
        active_pool
            .contract
            .methods()
            .get_asset(asset_id.into())
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_usdm_debt<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        active_pool
            .contract
            .methods()
            .increase_usdm_debt(amount, asset_id.into())
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdm_debt<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        active_pool
            .contract
            .methods()
            .decrease_usdm_debt(amount, asset_id.into())
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        asset_id: AssetId,
        trove_manager: Identity,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        active_pool
            .contract
            .methods()
            .add_asset(asset_id.into(), trove_manager)
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        token: &Token<T>,
        amount: u64,
    ) -> CallResponse<()> {
        let mock_asset_id = token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(mock_asset_id);

        active_pool
            .contract
            .methods()
            .recieve()
            .call_params(call_params)
            .unwrap()
            .with_contracts(&[token])
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(2))
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        recipient: Identity,
        amount: u64,
        asset_id: AssetId,
    ) -> CallResponse<()> {
        active_pool
            .contract
            .methods()
            .send_asset(recipient, amount, asset_id.into())
            .with_contract_ids(&[
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_default_pool<T: Account>(
        active_pool: &ContractInstance<ActivePool<T>>,
        default_pool: &ContractInstance<DefaultPool<T>>,
        asset: &Token<T>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        active_pool
            .contract
            .methods()
            .send_asset_to_default_pool(
                amount,
                asset
                    .contract_id()
                    .asset_id(&AssetId::zeroed().into())
                    .into(),
            )
            .with_contracts(&[&default_pool.contract, asset])
            .with_contract_ids(&[
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }
}
