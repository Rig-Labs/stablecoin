use crate::interfaces::active_pool::ActivePool;
use crate::interfaces::coll_surplus_pool::CollSurplusPool;
use crate::interfaces::community_issuance::CommunityIssuance;
use crate::interfaces::default_pool::DefaultPool;
use crate::interfaces::oracle::Oracle;
use crate::interfaces::pyth_oracle::PythCore;
use crate::interfaces::redstone_oracle::RedstoneCore;
use crate::interfaces::sorted_troves::SortedTroves;
use crate::interfaces::stability_pool::StabilityPool;
use crate::interfaces::usdf_token::USDFToken;
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "TroveManagerContract",
    abi = "contracts/trove-manager-contract/out/debug/trove-manager-contract-abi.json"
));

pub mod trove_manager_abi {

    use fuels::{
        prelude::{Account, Error, TxPolicies},
        types::{transaction_builders::VariableOutputPolicy, AssetId, ContractId, Identity},
    };

    use crate::data_structures::ContractInstance;

    use super::*;

    pub async fn get_nominal_icr<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        trove_manager
            .contract
            .methods()
            .get_nominal_icr(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn batch_liquidate_troves<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        community_issuance: &ContractInstance<CommunityIssuance<T>>,
        stability_pool: &ContractInstance<StabilityPool<T>>,
        oracle: &ContractInstance<Oracle<T>>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        default_pool: &ContractInstance<DefaultPool<T>>,
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        usdf: &ContractInstance<USDFToken<T>>,
        ids: Vec<Identity>,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .batch_liquidate_troves(ids, upper_hint, lower_hint)
            .with_tx_policies(tx_params)
            .with_contracts(&[
                &stability_pool.contract,
                &oracle.contract,
                pyth,
                redstone,
                &sorted_troves.contract,
                &active_pool.contract,
                &default_pool.contract,
                &coll_surplus_pool.contract,
                &usdf.contract,
                &community_issuance.contract,
            ])
            .with_contract_ids(&[
                sorted_troves.contract.contract_id().into(),
                sorted_troves.implementation_id.into(),
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
                pyth.contract_id().into(),
                redstone.contract_id().into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
                usdf.contract.contract_id().into(),
                usdf.implementation_id.into(),
                community_issuance.contract.contract_id().into(),
                community_issuance.implementation_id.into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .call()
            .await
    }

    pub async fn liquidate<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        community_issuance: &ContractInstance<CommunityIssuance<T>>,
        stability_pool: &ContractInstance<StabilityPool<T>>,
        oracle: &ContractInstance<Oracle<T>>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        default_pool: &ContractInstance<DefaultPool<T>>,
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        usdf: &ContractInstance<USDFToken<T>>,
        id: Identity,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .liquidate(id, upper_hint, lower_hint)
            .with_tx_policies(tx_params)
            .with_contracts(&[
                &stability_pool.contract,
                &oracle.contract,
                pyth,
                redstone,
                &sorted_troves.contract,
                &active_pool.contract,
                &default_pool.contract,
                &coll_surplus_pool.contract,
                &usdf.contract,
                &community_issuance.contract,
            ])
            .with_contract_ids(&[
                sorted_troves.contract.contract_id().into(),
                sorted_troves.implementation_id.into(),
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
                pyth.contract_id().into(),
                redstone.contract_id().into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
                default_pool.contract.contract_id().into(),
                default_pool.implementation_id.into(),
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
                usdf.contract.contract_id().into(),
                usdf.implementation_id.into(),
                community_issuance.contract.contract_id().into(),
                community_issuance.implementation_id.into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .call()
            .await
    }

    pub async fn increase_trove_coll<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        amount: u64,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .increase_trove_coll(id, amount)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_trove_debt<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        amount: u64,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .increase_trove_debt(id, amount)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_trove_status<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        status: Status,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .set_trove_status(id, status)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn initialize<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        borrow_operations: ContractId,
        sorted_troves_id: ContractId,
        oracle_id: ContractId,
        stability_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        coll_surplus_pool: ContractId,
        usdf: ContractId,
        asset: AssetId,
        protocol_manager: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = trove_manager
            .contract
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
                asset.into(),
                protocol_manager,
            )
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_trove_coll<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .get_trove_coll(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_entire_debt_and_coll<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<(u64, u64, u64, u64)> {
        let tx_params = TxPolicies::default().with_tip(1);

        trove_manager
            .contract
            .methods()
            .get_entire_debt_and_coll(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_debt<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        trove_manager
            .contract
            .methods()
            .get_trove_debt(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_status<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> Result<CallResponse<Status>, Error> {
        trove_manager
            .contract
            .methods()
            .get_trove_status(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_pending_asset_reward<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        trove_manager
            .contract
            .methods()
            .get_pending_asset_rewards(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_usdf_reward<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        trove_manager
            .contract
            .methods()
            .get_pending_usdf_rewards(id)
            .with_contract_ids(&[
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub fn get_redemption_fee(asset_drawdown: u64) -> u64 {
        return asset_drawdown * 1 / 100;
    }
}

pub mod trove_manager_utils {
    use fuels::{
        prelude::Account,
        types::{AssetId, Identity},
    };

    use crate::{
        data_structures::ContractInstance, interfaces::sorted_troves::sorted_troves_abi,
        setup::common::assert_within_threshold,
    };

    use super::*;

    pub async fn set_coll_and_debt_insert<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        id: Identity,
        coll: u64,
        debt: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: AssetId,
    ) {
        trove_manager_abi::increase_trove_coll(trove_manager, id.clone(), coll).await;
        trove_manager_abi::increase_trove_debt(trove_manager, id.clone(), debt).await;
        trove_manager_abi::set_trove_status(trove_manager, id.clone(), Status::Active).await;
        sorted_troves_abi::insert(sorted_troves, id, coll, prev_id, next_id, asset).await;
    }

    pub async fn assert_trove_coll<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        expected_coll: u64,
    ) {
        let real_coll = trove_manager_abi::get_trove_coll(&trove_manager, id)
            .await
            .value;

        assert_eq!(real_coll, expected_coll);
    }

    pub async fn assert_trove_debt<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        expected_debt: u64,
    ) {
        let real_debt = trove_manager_abi::get_trove_debt(&trove_manager, id)
            .await
            .value;

        assert_eq!(real_debt, expected_debt, "Incorrect trove debt");
    }

    pub async fn assert_trove_status<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        id: Identity,
        expected_status: Status,
    ) {
        let real_status = trove_manager_abi::get_trove_status(&trove_manager, id)
            .await
            .unwrap()
            .value;

        assert_eq!(real_status, expected_status, "Incorrect trove status");
    }

    pub async fn assert_pending_asset_rewards<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
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

    pub async fn assert_pending_usdf_rewards<T: Account>(
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
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
