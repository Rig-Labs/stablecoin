use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "DefaultPool",
    abi = "contracts/default-pool-contract/out/debug/default-pool-contract-abi.json"
));

pub mod default_pool_abi {
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::token::Token;
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId},
        types::Identity,
    };

    use super::*;

    pub async fn initialize(
        default_pool: &DefaultPool,
        trove_manager: Identity,
        active_pool: ContractId,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .initialize(trove_manager, active_pool, asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_usdf_debt(default_pool: &DefaultPool) -> FuelCallResponse<u64> {
        default_pool.methods().get_usdf_debt().call().await.unwrap()
    }

    pub async fn get_asset(default_pool: &DefaultPool) -> FuelCallResponse<u64> {
        default_pool.methods().get_asset().call().await.unwrap()
    }

    pub async fn increase_usdf_debt(
        default_pool: &DefaultPool,
        amount: u64,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .increase_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt(
        default_pool: &DefaultPool,
        amount: u64,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .decrease_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve(
        default_pool: &DefaultPool,
        token: &Token,
        amount: u64,
    ) -> FuelCallResponse<()> {
        let fuel_asset_id = AssetId::from(*token.contract_id().hash());

        let call_params: CallParameters = CallParameters {
            amount,
            asset_id: fuel_asset_id,
            gas_forwarded: None,
        };

        default_pool
            .methods()
            .recieve()
            .call_params(call_params)
            .set_contracts(&[token])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_active_pool(
        default_pool: &DefaultPool,
        active_pool: &ActivePool,
        amount: u64,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .send_asset_to_active_pool(amount)
            .set_contracts(&[active_pool])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }
}
