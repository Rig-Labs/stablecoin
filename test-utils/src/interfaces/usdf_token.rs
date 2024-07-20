use fuels::{
    prelude::{abigen, AssetId},
    programs::responses::CallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "USDFToken",
    abi = "contracts/usdf-token-contract/out/debug/usdf-token-contract-abi.json"
));

pub mod usdf_token_abi {
    use super::*;
    use fuels::{
        prelude::{Account, CallParameters, Error, TxPolicies},
        types::{transaction_builders::VariableOutputPolicy, ContractId},
    };

    pub async fn initialize<T: Account>(
        instance: &USDFToken<T>,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrow_operations: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        instance
            .methods()
            .initialize(
                protocol_manager,
                stability_pool.clone(),
                borrow_operations.clone(),
            )
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn mint<T: Account>(
        instance: &USDFToken<T>,
        amount: u64,
        address: Identity,
    ) -> Result<CallResponse<()>, Error> {
        instance
            .methods()
            .mint(amount, address)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }

    pub async fn burn<T: Account>(
        usdf_token: &USDFToken<T>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(200000);
        let usdf_asset_id = usdf_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdf_asset_id);

        let call_handler = usdf_token
            .methods()
            .burn()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1));

        call_handler
            .call_params(call_params)
            .unwrap()
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn total_supply<T: Account>(instance: &USDFToken<T>) -> CallResponse<Option<u64>> {
        let usdf_asset_id = instance
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        instance
            .methods()
            .total_supply(usdf_asset_id)
            .call()
            .await
            .unwrap()
    }
}
