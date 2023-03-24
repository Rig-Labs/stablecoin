use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "ActivePool",
    abi = "contracts/active-pool-contract/out/debug/active-pool-contract-abi.json"
));

pub mod active_pool_abi {
    use crate::interfaces::default_pool::DefaultPool;
    use crate::interfaces::token::Token;
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, Error},
        types::Identity,
    };

    use super::*;

    pub async fn initialize(
        active_pool: &ActivePool,
        borrow_operations: Identity,
        trove_manager: Identity,
        stability_pool: Identity,
        asset_id: ContractId,
        default_pool: ContractId,
    ) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .initialize(
                borrow_operations,
                trove_manager,
                stability_pool,
                asset_id,
                default_pool,
            )
            .call()
            .await
            .unwrap()
    }

    pub async fn get_usdf_debt(active_pool: &ActivePool) -> FuelCallResponse<u64> {
        active_pool.methods().get_usdf_debt().call().await.unwrap()
    }

    pub async fn get_asset(active_pool: &ActivePool) -> FuelCallResponse<u64> {
        active_pool.methods().get_asset().call().await.unwrap()
    }

    pub async fn increase_usdf_debt(active_pool: &ActivePool, amount: u64) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .increase_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt(active_pool: &ActivePool, amount: u64) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .decrease_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve(
        active_pool: &ActivePool,
        token: &Token,
        amount: u64,
    ) -> FuelCallResponse<()> {
        let fuel_asset_id = AssetId::from(*token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(fuel_asset_id);

        active_pool
            .methods()
            .recieve()
            .call_params(call_params)
            .unwrap()
            .set_contracts(&[token])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset(
        active_pool: &ActivePool,
        recipient: Identity,
        amount: u64,
    ) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .send_asset(recipient, amount)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_default_pool(
        active_pool: &ActivePool,
        default_pool: &DefaultPool,
        asset: &Token,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        active_pool
            .methods()
            .send_asset_to_default_pool(amount)
            .set_contracts(&[default_pool, asset])
            .append_variable_outputs(1)
            .call()
            .await
    }
}
