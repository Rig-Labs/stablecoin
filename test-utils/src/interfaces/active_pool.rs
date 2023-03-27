use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "ActivePool",
    abi = "contracts/active-pool-contract/out/debug/active-pool-contract-abi.json"
));

pub mod active_pool_abi {
    use crate::interfaces::token::Token;
    use crate::{interfaces::default_pool::DefaultPool, setup::common::wait};
    use fuels::prelude::LogDecoder;
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, Error, TxParameters},
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
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = active_pool
            .methods()
            .initialize(
                borrow_operations.clone(),
                trove_manager.clone(),
                stability_pool.clone(),
                asset_id,
                default_pool,
            )
            .tx_params(tx_params)
            .call()
            .await;

        // TODO: remove this workaround
        match res {
            Ok(res) => res,
            Err(_) => {
                wait();
                return FuelCallResponse::new((), vec![], LogDecoder::default());
            }
        }
    }

    pub async fn get_usdf_debt(active_pool: &ActivePool) -> FuelCallResponse<u64> {
        active_pool.methods().get_usdf_debt().call().await.unwrap()
    }

    pub async fn get_asset(active_pool: &ActivePool, asset: ContractId) -> FuelCallResponse<u64> {
        active_pool.methods().get_asset(asset).call().await.unwrap()
    }

    pub async fn increase_usdf_debt(active_pool: &ActivePool, amount: u64) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        active_pool
            .methods()
            .increase_usdf_debt(amount)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt(active_pool: &ActivePool, amount: u64) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        active_pool
            .methods()
            .decrease_usdf_debt(amount)
            .tx_params(tx_params)
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
        asset: &Token,
        amount: u64,
    ) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .send_asset(recipient, asset.contract_id().into(), amount)
            .set_contracts(&[asset])
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
            .send_asset_to_default_pool(asset.contract_id().into(), amount)
            .set_contracts(&[default_pool, asset])
            .append_variable_outputs(1)
            .call()
            .await
    }
}
