use fuels::programs::call_utils::TxDependencyExtension;
use fuels::{
    prelude::abigen, prelude::BASE_ASSET_ID, programs::call_response::FuelCallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "USDFToken",
    abi = "contracts/usdf-token-contract/out/debug/usdf-token-contract-abi.json"
));

pub mod usdf_token_abi {
    use super::*;
    use fuels::{
        prelude::{Account, CallParameters, Error, TxParameters},
        types::ContractId,
    };

    pub async fn initialize<T: Account>(
        instance: &USDFToken<T>,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrow_operations: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().with_gas_price(1);

        instance
            .methods()
            .initialize(
                protocol_manager,
                stability_pool.clone(),
                borrow_operations.clone(),
            )
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn mint<T: Account>(
        instance: &USDFToken<T>,
        amount: u64,
        address: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        instance
            .methods()
            .mint(amount, address)
            .append_variable_outputs(1)
            .call()
            .await
    }

    pub async fn burn<T: Account>(
        usdf_token: &USDFToken<T>,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().with_gas_price(1);
        let usdf_asset_id = usdf_token
            .contract_id()
            .asset_id(&BASE_ASSET_ID.into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdf_asset_id);

        let call_handler = usdf_token.methods().burn().append_variable_outputs(1);

        call_handler
            .call_params(call_params)
            .unwrap()
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn total_supply<T: Account>(
        instance: &USDFToken<T>,
    ) -> FuelCallResponse<Option<u64>> {
        let usdf_asset_id = instance
            .contract_id()
            .asset_id(&BASE_ASSET_ID.into())
            .into();

        instance
            .methods()
            .total_supply(usdf_asset_id)
            .call()
            .await
            .unwrap()
    }
}
