use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

use crate::interfaces::active_pool::ActivePool;
use crate::interfaces::coll_surplus_pool::CollSurplusPool;
use crate::interfaces::default_pool::DefaultPool;
use crate::interfaces::oracle::Oracle;
use crate::interfaces::sorted_troves::SortedTroves;
use crate::interfaces::stability_pool::StabilityPool;
use crate::interfaces::token::Token;
use crate::interfaces::usdf_token::USDFToken;

abigen!(Contract(
    name = "TroveManagerContract",
    abi = "contracts/trove-manager-contract/out/debug/trove-manager-contract-abi.json"
));

pub mod trove_manager_abi {

    use fuels::prelude::{AssetId, CallParameters, Error};

    use super::*;

    pub async fn get_nominal_icr(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_nominal_icr(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn liquidate(
        trove_manager: &TroveManagerContract,
        stability_pool: &StabilityPool,
        oracle: &Oracle,
        sorted_troves: &SortedTroves,
        active_pool: &ActivePool,
        default_pool: &DefaultPool,
        coll_surplus_pool: &CollSurplusPool,
        usdf: &USDFToken,
        id: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .liquidate(id)
            .tx_params(tx_params)
            .set_contracts(&[
                stability_pool,
                oracle,
                sorted_troves,
                active_pool,
                default_pool,
                coll_surplus_pool,
                usdf,
            ])
            .append_variable_outputs(3)
            .call()
            .await
    }

    pub async fn increase_trove_coll(
        trove_manager: &TroveManagerContract,
        id: Identity,
        amount: u64,
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .increase_trove_coll(id, amount)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_trove_debt(
        trove_manager: &TroveManagerContract,
        id: Identity,
        amount: u64,
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .increase_trove_debt(id, amount)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_trove_status(
        trove_manager: &TroveManagerContract,
        id: Identity,
        status: Status,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .set_trove_status(id, status)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn remove(
        trove_manager: &TroveManagerContract,
        sorted_troves: &SortedTroves,
        id: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .remove(id)
            .set_contracts(&[sorted_troves])
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn initialize(
        trove_manager: &TroveManagerContract,
        borrow_operations: ContractId,
        sorted_troves_id: ContractId,
        oracle_id: ContractId,
        stability_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        coll_surplus_pool: ContractId,
        usdf: ContractId,
    ) -> FuelCallResponse<()> {
        trove_manager
            .methods()
            .initialize(
                borrow_operations,
                sorted_troves_id,
                oracle_id,
                stability_pool,
                default_pool,
                active_pool,
                coll_surplus_pool,
                usdf,
            )
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_coll(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_trove_coll(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_debt(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_trove_debt(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_status(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> Result<FuelCallResponse<Status>, Error> {
        trove_manager.methods().get_trove_status(id).call().await
    }

    pub async fn get_pending_asset_reward(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_pending_asset_rewards(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_usdf_reward(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_pending_usdf_rewards(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn redeem_collateral(
        trove_manager: &TroveManagerContract,
        amount: u64,
        max_iterations: u64,
        max_fee_percentage: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Option<Identity>,
        lower_partial_hint: Option<Identity>,
        usdf: &USDFToken,
        fuel: &Token,
        sorted_troves: &SortedTroves,
        active_pool: &ActivePool,
        coll_surplus_pool: &CollSurplusPool,
        oracle: &Oracle,
        default_pool: &DefaultPool,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(20_000_000), Some(0));
        let usdf_asset_id = AssetId::from(*usdf.contract_id().hash());

        let call_params: CallParameters = CallParameters {
            amount,
            asset_id: usdf_asset_id,
            gas_forwarded: None,
        };

        trove_manager
            .methods()
            .redeem_collateral(
                max_iterations,
                max_fee_percentage,
                partial_redemption_hint,
                upper_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
                lower_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
            )
            .tx_params(tx_params)
            .call_params(call_params)
            .set_contracts(&[
                sorted_troves,
                active_pool,
                fuel,
                usdf,
                coll_surplus_pool,
                oracle,
                default_pool,
            ])
            .append_variable_outputs(10)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_borrowing_rate(trove_manager: &TroveManagerContract) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_borrowing_rate()
            .call()
            .await
            .unwrap()
    }

    pub async fn get_borrowing_rate_with_decay(
        trove_manager: &TroveManagerContract,
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::new(Some(1), Some(2_000_000), Some(0));

        trove_manager
            .methods()
            .get_borrowing_rate_with_decay()
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_borrowing_fee(
        trove_manager: &TroveManagerContract,
        usdf_borrowed: u64,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_borrowing_fee(usdf_borrowed)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_borrowing_fee_with_decay(
        trove_manager: &TroveManagerContract,
        usdf_borrowed: u64,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_borrowing_fee_with_decay(usdf_borrowed)
            .call()
            .await
            .unwrap()
    }
}

pub mod trove_manager_utils {
    use crate::{
        interfaces::sorted_troves::sorted_troves_abi, setup::common::assert_within_threshold,
    };

    use super::*;

    pub async fn set_coll_and_debt_insert(
        trove_manager: &TroveManagerContract,
        sorted_troves: &SortedTroves,
        id: Identity,
        coll: u64,
        debt: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        trove_manager_abi::increase_trove_coll(trove_manager, id.clone(), coll).await;
        trove_manager_abi::increase_trove_debt(trove_manager, id.clone(), debt).await;
        trove_manager_abi::set_trove_status(trove_manager, id.clone(), Status::Active).await;
        sorted_troves_abi::insert(sorted_troves, id, coll, prev_id, next_id).await;
    }

    pub async fn assert_trove_coll(
        trove_manager: &TroveManagerContract,
        id: Identity,
        expected_coll: u64,
    ) {
        let real_coll = trove_manager_abi::get_trove_coll(&trove_manager, id)
            .await
            .value;

        assert_eq!(real_coll, expected_coll);
    }

    pub async fn assert_trove_debt(
        trove_manager: &TroveManagerContract,
        id: Identity,
        expected_debt: u64,
    ) {
        let real_debt = trove_manager_abi::get_trove_debt(&trove_manager, id)
            .await
            .value;

        assert_eq!(real_debt, expected_debt, "Incorrect trove debt");
    }

    pub async fn assert_trove_status(
        trove_manager: &TroveManagerContract,
        id: Identity,
        expected_status: Status,
    ) {
        let real_status = trove_manager_abi::get_trove_status(&trove_manager, id)
            .await
            .unwrap()
            .value;

        assert_eq!(real_status, expected_status, "Incorrect trove status");
    }

    pub async fn assert_pending_asset_rewards(
        trove_manager: &TroveManagerContract,
        id: Identity,
        expected_rewards: u64,
    ) {
        let real_rewards = trove_manager_abi::get_pending_asset_reward(&trove_manager, id)
            .await
            .value;

        assert_within_threshold(
            real_rewards,
            expected_rewards,
            &format!(
                "Rewards are not within 0.001% threshold, expected: {}, real: {}",
                expected_rewards, real_rewards
            ),
        );
    }

    pub async fn assert_pending_usdf_rewards(
        trove_manager: &TroveManagerContract,
        id: Identity,
        expected_rewards: u64,
    ) {
        let real_rewards = trove_manager_abi::get_pending_usdf_reward(&trove_manager, id)
            .await
            .value;

        assert_within_threshold(
            real_rewards,
            expected_rewards,
            &format!(
                "USDF Rewards are not within 0.001% threshold, expected: {}, real: {}",
                expected_rewards, real_rewards
            ),
        );
    }
}
