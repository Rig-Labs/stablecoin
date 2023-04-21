use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "DefaultPool",
    abi = "contracts/default-pool-contract/out/debug/default-pool-contract-abi.json"
));

pub mod default_pool_abi {
    use crate::interfaces::token::Token;
    use crate::{interfaces::active_pool::ActivePool, setup::common::wait};
    use fuels::prelude::{Account, LogDecoder};
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize<T: Account>(
        default_pool: &DefaultPool<T>,
        trove_manager: Identity,
        active_pool: ContractId,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = default_pool
            .methods()
            .initialize(trove_manager.clone(), active_pool, asset_id)
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

    pub async fn get_usdf_debt<T: Account>(default_pool: &DefaultPool<T>) -> FuelCallResponse<u64> {
        default_pool.methods().get_usdf_debt().call().await.unwrap()
    }

    pub async fn get_asset<T: Account>(default_pool: &DefaultPool<T>) -> FuelCallResponse<u64> {
        default_pool.methods().get_asset().call().await.unwrap()
    }

    pub async fn increase_usdf_debt<T: Account>(
        default_pool: &DefaultPool<T>,
        amount: u64,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .increase_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt<T: Account>(
        default_pool: &DefaultPool<T>,
        amount: u64,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .decrease_usdf_debt(amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve<T: Account>(
        default_pool: &DefaultPool<T>,
        token: &Token<T>,
        amount: u64,
    ) -> FuelCallResponse<()> {
        let fuel_asset_id = AssetId::from(*token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(fuel_asset_id);

        let tx_params = TxParameters::default().set_gas_price(1);

        default_pool
            .methods()
            .recieve()
            .tx_params(tx_params)
            .append_variable_outputs(1)
            .call_params(call_params)
            .unwrap()
            .set_contracts(&[token])
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_active_pool<T: Account>(
        default_pool: &DefaultPool<T>,
        active_pool: &ActivePool<T>,
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
