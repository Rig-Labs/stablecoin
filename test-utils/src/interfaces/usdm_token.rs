use fuels::{
    prelude::{abigen, AssetId},
    programs::responses::CallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "USDMToken",
    abi = "contracts/usdm-token-contract/out/debug/usdm-token-contract-abi.json"
));

pub mod usdm_token_abi {
    use crate::data_structures::ContractInstance;

    use super::*;
    use fuels::{
        prelude::{Account, CallParameters, Error, TxPolicies},
        types::{transaction_builders::VariableOutputPolicy, Bits256, ContractId},
    };

    pub async fn initialize<T: Account>(
        instance: &ContractInstance<USDMToken<T>>,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrow_operations: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        instance
            .contract
            .methods()
            .initialize(
                protocol_manager,
                stability_pool.clone(),
                borrow_operations.clone(),
            )
            .with_contract_ids(&[
                instance.implementation_id.into(),
                instance.contract.contract_id().into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn mint<T: Account>(
        instance: &ContractInstance<USDMToken<T>>,
        amount: u64,
        address: Identity,
    ) -> Result<CallResponse<()>, Error> {
        instance
            .contract
            .methods()
            .mint(address, None, amount)
            .with_contract_ids(&[
                instance.implementation_id.into(),
                instance.contract.contract_id().into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }

    pub async fn burn<T: Account>(
        instance: &ContractInstance<USDMToken<T>>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(200000);
        let usdm_asset_id = instance
            .contract
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdm_asset_id);

        let call_handler = instance
            .contract
            .methods()
            .burn(Bits256::zeroed(), amount)
            .with_contract_ids(&[
                instance.implementation_id.into(),
                instance.contract.contract_id().into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1));

        call_handler
            .call_params(call_params)
            .unwrap()
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn total_supply<T: Account>(
        instance: &ContractInstance<USDMToken<T>>,
    ) -> CallResponse<Option<u64>> {
        let usdm_asset_id = instance
            .contract
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        instance
            .contract
            .methods()
            .total_supply(usdm_asset_id)
            .with_contract_ids(&[
                instance.implementation_id.into(),
                instance.contract.contract_id().into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn add_trove_manager<T: Account>(
        instance: &ContractInstance<USDMToken<T>>,
        trove_manager: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        instance
            .contract
            .methods()
            .add_trove_manager(trove_manager)
            .with_contract_ids(&[
                instance.implementation_id.into(),
                instance.contract.contract_id().into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
    }
}
