use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
};

use crate::interfaces::active_pool::ActivePool;
use crate::interfaces::borrow_operations::BorrowOperations;
use crate::interfaces::oracle::Oracle;
use crate::interfaces::sorted_troves::SortedTroves;
use crate::interfaces::token::Token;
use crate::interfaces::trove_manager::TroveManagerContract;
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
        let tx_params = TxParameters::default().set_gas_price(1);

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
        fuel_token: &Token,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let usdf_asset_id = AssetId::from(*usdf_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(usdf_asset_id);

        stability_pool
            .methods()
            .provide_to_stability_pool()
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .append_variable_outputs(2)
            .set_contracts(&[usdf_token, fuel_token])
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
        let tx_params = TxParameters::default().set_gas_price(1);

        stability_pool
            .methods()
            .withdraw_from_stability_pool(amount)
            .tx_params(tx_params)
            .append_variable_outputs(2)
            .set_contracts(&[usdf_token, fuel_token])
            .call()
            .await
    }

    pub async fn withdraw_gain_to_trove(
        stability_pool: &StabilityPool,
        usdf_token: &USDFToken,
        fuel_token: &Token,
        trove_manager: &TroveManagerContract,
        borrow_operations: &BorrowOperations,
        sorted_troves: &SortedTroves,
        active_pool: &ActivePool,
        oracle: &Oracle,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);

        stability_pool
            .methods()
            .withdraw_gain_to_trove(lower_hint, upper_hint)
            .tx_params(tx_params)
            .append_variable_outputs(2)
            .set_contracts(&[
                usdf_token,
                fuel_token,
                trove_manager,
                borrow_operations,
                sorted_troves,
                active_pool,
                oracle,
            ])
            .call()
            .await
    }
}

pub mod stability_pool_utils {
    use fuels::types::Identity;

    use crate::setup::common::assert_within_threshold;

    use super::*;

    pub async fn assert_pool_asset(stability_pool: &StabilityPool, expected_asset_amount: u64) {
        let pool_asset = super::stability_pool_abi::get_asset(stability_pool)
            .await
            .unwrap()
            .value;

        assert_eq!(pool_asset, expected_asset_amount);
    }

    pub async fn assert_total_usdf_deposits(
        stability_pool: &StabilityPool,
        expected_usdf_amount: u64,
    ) {
        let total_usdf_deposits =
            super::stability_pool_abi::get_total_usdf_deposits(stability_pool)
                .await
                .unwrap()
                .value;

        assert_eq!(total_usdf_deposits, expected_usdf_amount);
    }

    pub async fn assert_depositor_asset_gain(
        stability_pool: &StabilityPool,
        depositor: Identity,
        expected_asset_gain: u64,
    ) {
        let depositor_asset_gain =
            super::stability_pool_abi::get_depositor_asset_gain(stability_pool, depositor)
                .await
                .unwrap()
                .value;

        assert_within_threshold(
            expected_asset_gain,
            depositor_asset_gain,
            &format!(
                "Depsoitor gains not within 0.001% threshold, expected: {}, real: {}",
                expected_asset_gain, depositor_asset_gain
            ),
        );
    }

    pub async fn assert_compounded_usdf_deposit(
        stability_pool: &StabilityPool,
        depositor: Identity,
        expected_compounded_usdf_deposit: u64,
    ) {
        let compounded_usdf_deposit =
            super::stability_pool_abi::get_compounded_usdf_deposit(stability_pool, depositor)
                .await
                .unwrap()
                .value;

        assert_within_threshold(
            expected_compounded_usdf_deposit,
            compounded_usdf_deposit,
            &format!(
                "Compounded USDF deposit not within 0.001% threshold, expected: {}, real: {}",
                expected_compounded_usdf_deposit, compounded_usdf_deposit
            ),
        );
    }
}
