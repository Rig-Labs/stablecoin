use fuels::prelude::abigen;
use fuels::programs::call_response::FuelCallResponse;
use fuels::programs::call_utils::TxDependencyExtension;

abigen!(Contract(
    name = "DefaultPool",
    abi = "contracts/default-pool-contract/out/debug/default-pool-contract-abi.json"
));

pub mod default_pool_abi {
    use super::*;
    use crate::interfaces::token::Token;
    use crate::{interfaces::active_pool::ActivePool, setup::common::wait};
    use fuels::prelude::{Account, LogDecoder};
    use fuels::{
        prelude::{AssetId, CallParameters, ContractId, Error, TxParameters},
        types::Identity,
    };

    pub async fn initialize<T: Account>(
        default_pool: &DefaultPool<T>,
        protocol_manager: Identity,
        active_pool: ContractId,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = default_pool
            .methods()
            .initialize(protocol_manager.clone(), active_pool)
            .tx_params(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_usdf_debt<T: Account>(
        default_pool: &DefaultPool<T>,
        asset_id: ContractId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_usdf_debt(asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_asset<T: Account>(
        default_pool: &DefaultPool<T>,
        asset_id: ContractId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_asset(asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_usdf_debt<T: Account>(
        default_pool: &DefaultPool<T>,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .increase_usdf_debt(amount, asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        default_pool: &DefaultPool<T>,
        asset_id: ContractId,
        trove_manager: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        default_pool
            .methods()
            .add_asset(asset_id, trove_manager)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn decrease_usdf_debt<T: Account>(
        default_pool: &DefaultPool<T>,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .decrease_usdf_debt(amount, asset_id)
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
            .with_amount(amount)
            .with_asset_id(fuel_asset_id);

        let tx_params = TxParameters::default().with_gas_price(1);

        default_pool
            .methods()
            .recieve()
            .tx_params(tx_params)
            .append_variable_outputs(1)
            .call_params(call_params)
            .unwrap()
            .with_contracts(&[token])
            .call()
            .await
            .unwrap()
    }

    pub async fn send_asset_to_active_pool<T: Account>(
        default_pool: &DefaultPool<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .send_asset_to_active_pool(amount, asset_id)
            .with_contracts(&[active_pool])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }
}
