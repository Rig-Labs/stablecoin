use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
};

use crate::interfaces::token::Token;
use crate::interfaces::usdf_token::USDFToken;
abigen!(Contract(
    name = "StabilityPool",
    abi = "contracts/stability-pool-contract/out/debug/stability-pool-contract-abi.json"
));

pub mod stability_pool_abi {
    use fuels::{
        prelude::{AssetId, CallParameters, Error},
        types::Identity,
    };

    use super::*;

    pub async fn initialize(
        stability_pool: &StabilityPool,
        borrow_operations_address: ContractId,
        trove_manager_address: ContractId,
        active_pool_address: ContractId,
        usdf_address: ContractId,
        sorted_troves_address: ContractId,
        oracle_address: ContractId,
        community_issuance_address: ContractId,
        asset_address: ContractId,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        stability_pool
            .methods()
            .initialize(
                borrow_operations_address,
                trove_manager_address,
                active_pool_address,
                usdf_address,
                sorted_troves_address,
                oracle_address,
                community_issuance_address,
                asset_address,
            )
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn provide_to_stability_pool(
        stability_pool: &StabilityPool,
        usdf_token: &USDFToken,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        let usdf_asset_id = AssetId::from(*usdf_token.contract_id().hash());

        let call_params: CallParameters = CallParameters {
            amount,
            asset_id: usdf_asset_id,
            gas_forwarded: None,
        };

        stability_pool
            .methods()
            .provide_to_stability_pool()
            .tx_params(tx_params)
            .call_params(call_params)
            .set_contracts(&[usdf_token])
            .call()
            .await
    }

    pub async fn get_asset(stability_pool: &StabilityPool) -> Result<FuelCallResponse<u64>, Error> {
        stability_pool.methods().get_asset().call().await
    }

    pub async fn get_total_usdf_deposits(
        stability_pool: &StabilityPool,
    ) -> Result<FuelCallResponse<u64>, Error> {
        stability_pool
            .methods()
            .get_total_usdf_deposits()
            .call()
            .await
    }

    pub async fn get_depositor_asset_gain(
        stability_pool: &StabilityPool,
        depositor: Identity,
    ) -> Result<FuelCallResponse<u64>, Error> {
        stability_pool
            .methods()
            .get_depositor_asset_gain(depositor)
            .call()
            .await
    }

    pub async fn get_compounded_usdf_deposit(
        stability_pool: &StabilityPool,
        depositor: Identity,
    ) -> Result<FuelCallResponse<u64>, Error> {
        stability_pool
            .methods()
            .get_compounded_usdf_deposit(depositor)
            .call()
            .await
    }

    pub async fn withdraw_from_stability_pool(
        stability_pool: &StabilityPool,
        usdf_token: &USDFToken,
        fuel_token: &Token,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        stability_pool
            .methods()
            .withdraw_from_stability_pool(amount)
            .tx_params(tx_params)
            .append_variable_outputs(2)
            .set_contracts(&[usdf_token, fuel_token])
            .call()
            .await
    }
}

pub mod stability_pool_utils {}
