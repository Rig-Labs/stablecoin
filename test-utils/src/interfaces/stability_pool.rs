use fuels::prelude::abigen;

use crate::interfaces::community_issuance::CommunityIssuance;
use crate::interfaces::oracle::Oracle;
use crate::interfaces::pyth_oracle::PythCore;
use crate::interfaces::redstone_oracle::RedstoneCore;
use crate::interfaces::sorted_troves::SortedTroves;
use crate::interfaces::token::Token;
use crate::interfaces::trove_manager::TroveManagerContract;
use crate::interfaces::usdm_token::USDMToken;

abigen!(Contract(
    name = "StabilityPool",
    abi = "contracts/stability-pool-contract/out/debug/stability-pool-contract-abi.json"
));

pub mod stability_pool_abi {

    use crate::data_structures::ContractInstance;

    use super::*;
    use fuels::{
        prelude::{Account, CallParameters, Error, TxPolicies},
        programs::responses::CallResponse,
        types::{transaction_builders::VariableOutputPolicy, AssetId, ContractId, Identity},
    };

    pub async fn initialize<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        usdm_address: ContractId,
        community_issuance_address: ContractId,
        protocol_manager_contract: ContractId,
        active_pool: ContractId,
        sorted_troves: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        stability_pool
            .contract
            .methods()
            .initialize(
                usdm_address,
                community_issuance_address,
                protocol_manager_contract,
                active_pool,
                sorted_troves,
            )
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn add_asset<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        trove_manager: ContractId,
        asset_address: AssetId,
        oracle_address: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        stability_pool
            .contract
            .methods()
            .add_asset(trove_manager, asset_address.into(), oracle_address)
            .with_tx_policies(tx_params)
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn provide_to_stability_pool<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        community_issuance: &ContractInstance<CommunityIssuance<T>>,
        usdm_token: &ContractInstance<USDMToken<T>>,
        mock_token: &Token<T>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let usdm_asset_id = usdm_token
            .contract
            .contract_id()
            .asset_id(&AssetId::zeroed().into());

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdm_asset_id);

        stability_pool
            .contract
            .methods()
            .provide_to_stability_pool()
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(2))
            .with_contracts(&[
                &usdm_token.contract,
                mock_token,
                &community_issuance.contract,
            ])
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
                usdm_token.contract.contract_id().into(),
                usdm_token.implementation_id.into(),
                mock_token.contract_id().into(),
                community_issuance.contract.contract_id().into(),
                community_issuance.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_asset<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        asset_address: AssetId,
    ) -> Result<CallResponse<u64>, Error> {
        stability_pool
            .contract
            .methods()
            .get_asset(asset_address)
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_total_usdm_deposits<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
    ) -> Result<CallResponse<u64>, Error> {
        stability_pool
            .contract
            .methods()
            .get_total_usdm_deposits()
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_depositor_asset_gain<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        depositor: Identity,
        asset_id: AssetId,
    ) -> Result<CallResponse<u64>, Error> {
        stability_pool
            .contract
            .methods()
            .get_depositor_asset_gain(depositor, asset_id.into())
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_compounded_usdm_deposit<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        depositor: Identity,
    ) -> Result<CallResponse<u64>, Error> {
        stability_pool
            .contract
            .methods()
            .get_compounded_usdm_deposit(depositor)
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn get_depositor_fpt_gain<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        depositor: Identity,
    ) -> Result<CallResponse<u64>, Error> {
        stability_pool
            .contract
            .methods()
            .get_depositor_fpt_gain(depositor)
            .with_contract_ids(&[
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn withdraw_from_stability_pool<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        community_issuance: &ContractInstance<CommunityIssuance<T>>,
        usdm_token: &ContractInstance<USDMToken<T>>,
        mock_token: &Token<T>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        oracle: &ContractInstance<Oracle<T>>,
        pyth_oracle: &PythCore<T>,
        _redstone_oracle: &RedstoneCore<T>,
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        stability_pool
            .contract
            .methods()
            .withdraw_from_stability_pool(amount)
            .with_tx_policies(tx_params)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(2))
            .with_contracts(&[
                &usdm_token.contract,
                mock_token,
                &community_issuance.contract,
                &sorted_troves.contract,
                &oracle.contract,
                pyth_oracle,
                // redstone_oracle,
                &trove_manager.contract,
            ])
            .with_contract_ids(&[
                sorted_troves.contract.contract_id().into(),
                sorted_troves.implementation_id.into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
                pyth_oracle.contract_id().into(),
                // redstone_oracle.contract_id().into(),
                usdm_token.contract.contract_id().into(),
                usdm_token.implementation_id.into(),
                mock_token.contract_id().into(),
                community_issuance.contract.contract_id().into(),
                community_issuance.implementation_id.into(),
                stability_pool.contract.contract_id().into(),
                stability_pool.implementation_id.into(),
            ])
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await
    }
}

pub mod stability_pool_utils {
    use fuels::{
        prelude::Account,
        types::{AssetId, Identity},
    };

    use crate::{data_structures::ContractInstance, setup::common::assert_within_threshold};

    use super::*;

    pub async fn assert_pool_asset<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        expected_asset_amount: u64,
        asset_address: AssetId,
    ) {
        let pool_asset = super::stability_pool_abi::get_asset(stability_pool, asset_address.into())
            .await
            .unwrap()
            .value;

        assert_eq!(pool_asset, expected_asset_amount);
    }

    pub async fn assert_total_usdm_deposits<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        expected_usdm_amount: u64,
    ) {
        let total_usdm_deposits =
            super::stability_pool_abi::get_total_usdm_deposits(stability_pool)
                .await
                .unwrap()
                .value;

        assert_eq!(total_usdm_deposits, expected_usdm_amount);
    }

    pub async fn assert_depositor_asset_gain<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        depositor: Identity,
        expected_asset_gain: u64,
        asset_address: AssetId,
    ) {
        let depositor_asset_gain = super::stability_pool_abi::get_depositor_asset_gain(
            stability_pool,
            depositor,
            asset_address,
        )
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

    pub async fn assert_compounded_usdm_deposit<T: Account + Clone>(
        stability_pool: &ContractInstance<StabilityPool<T>>,
        depositor: Identity,
        expected_compounded_usdm_deposit: u64,
    ) {
        let compounded_usdm_deposit =
            stability_pool_abi::get_compounded_usdm_deposit(stability_pool, depositor)
                .await
                .unwrap()
                .value;

        assert_within_threshold(
            expected_compounded_usdm_deposit,
            compounded_usdm_deposit,
            &format!(
                "Compounded USDM deposit not within 0.001% threshold, expected: {}, real: {}",
                expected_compounded_usdm_deposit, compounded_usdm_deposit
            ),
        );
    }
}
