use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "ActivePool",
    abi = "contracts/active-pool-contract/out/debug/active-pool-contract-abi.json"
));

pub mod active_pool_abi {
    use crate::interfaces::token::Token;
    use crate::{interfaces::default_pool::DefaultPool, setup::common::wait};
    use fuels::prelude::{Account, LogDecoder};
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, Error, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize<T: Account>(
        active_pool: &ActivePool<T>,
        borrow_operations: Identity,
        stability_pool: Identity,
        default_pool: ContractId,
        protocol_manager: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = active_pool
            .methods()
            .initialize(
                borrow_operations.clone(),
                stability_pool.clone(),
                default_pool,
                protocol_manager,
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

    pub async fn get_usdf_debt<T: Account>(
        active_pool: &ActivePool<T>,
        asset_id: ContractId,
    ) -> FuelCallResponse<u64> {
        active_pool
            .methods()
            .get_usdf_debt(asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_asset<T: Account>(
        active_pool: &ActivePool<T>,
        asset_id: ContractId,
    ) -> FuelCallResponse<u64> {
        active_pool
            .methods()
            .get_asset(asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_usdf_debt<T: Account>(
        active_pool: &ActivePool<T>,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        active_pool
            .methods()
            .increase_usdf_debt(amount, asset_id)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt<T: Account>(
        active_pool: &ActivePool<T>,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        active_pool
            .methods()
            .decrease_usdf_debt(amount, asset_id)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        active_pool: &ActivePool<T>,
        asset_id: ContractId,
        trove_manager: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        active_pool
            .methods()
            .add_asset(asset_id, trove_manager)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn recieve<T: Account>(
        active_pool: &ActivePool<T>,
        token: &Token<T>,
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

    pub async fn send_asset<T: Account>(
        active_pool: &ActivePool<T>,
        recipient: Identity,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        active_pool
            .methods()
            .send_asset(recipient, amount, asset_id)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_default_pool<T: Account>(
        active_pool: &ActivePool<T>,
        default_pool: &DefaultPool<T>,
        asset: &Token<T>,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        active_pool
            .methods()
            .send_asset_to_default_pool(amount, asset.contract_id().into())
            .set_contracts(&[default_pool, asset])
            .append_variable_outputs(1)
            .call()
            .await
    }
}
