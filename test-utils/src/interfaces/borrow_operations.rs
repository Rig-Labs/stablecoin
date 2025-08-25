use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "BorrowOperations",
    abi = "contracts/borrow-operations-contract/out/debug/borrow-operations-contract-abi.json"
));

pub mod borrow_operations_abi {
    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::coll_surplus_pool::CollSurplusPool;
    use crate::interfaces::default_pool::DefaultPool;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::oracle::Oracle;
    use crate::interfaces::pyth_oracle::PythCore;
    use crate::interfaces::redstone_oracle::RedstoneCore;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::token::Token;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use crate::interfaces::usdm_token::USDMToken;
    use fuels::prelude::Account;
    use fuels::prelude::{CallParameters, ContractId, Error, TxPolicies};
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::types::{AssetId, Identity};

    pub async fn initialize<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        usdm_contract: ContractId,
        fpt_staking_contract: ContractId,
        protocol_manager_contract: ContractId,
        coll_surplus_pool_contract: ContractId,
        active_pool_contract: ContractId,
        sorted_troves_contract: ContractId,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .initialize(
                usdm_contract,
                fpt_staking_contract,
                protocol_manager_contract,
                coll_surplus_pool_contract,
                active_pool_contract,
                sorted_troves_contract,
            )
            .with_tx_policies(tx_params)
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn open_trove<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        oracle: &ContractInstance<Oracle<T>>,
        mock_pyth: &PythCore<T>,
        _mock_redstone: &RedstoneCore<T>,
        asset_token: &Token<T>,
        usdm_token: &ContractInstance<USDMToken<T>>,
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        collateral_amount_deposit: u64,
        usdm_amount_withdrawn: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let asset_id = asset_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(collateral_amount_deposit)
            .with_asset_id(asset_id);

        return borrow_operations
            .contract
            .methods()
            .open_trove(usdm_amount_withdrawn, upper_hint, lower_hint)
            .call_params(call_params)
            .unwrap()
            .with_contracts(&[
                &oracle.contract,
                mock_pyth,
                //mock_redstone,
                &active_pool.contract,
                &usdm_token.contract,
                &sorted_troves.contract,
                &trove_manager.contract,
                &fpt_staking.contract,
            ])
            .with_contract_ids(&[
                borrow_operations.contract.contract_id().into(),
                borrow_operations.implementation_id.into(),
                sorted_troves.implementation_id.into(),
                sorted_troves.contract.contract_id().into(),
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
                mock_pyth.contract_id().into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
                usdm_token.contract.contract_id().into(),
                usdm_token.implementation_id.into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .with_tx_policies(tx_params)
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await;
    }

    pub async fn add_coll<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        _oracle: &ContractInstance<Oracle<T>>,
        _pyth: &PythCore<T>,
        _redstone: &RedstoneCore<T>,
        mock_token: &Token<T>,
        _usdm_token: &ContractInstance<USDMToken<T>>,
        _sorted_troves: &ContractInstance<SortedTroves<T>>,
        _trove_manager: &ContractInstance<TroveManagerContract<T>>,
        _active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let mock_asset_id: AssetId = mock_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(mock_asset_id);

        borrow_operations
            .contract
            .methods()
            .add_coll(lower_hint, upper_hint)
            .call_params(call_params)
            .unwrap()
            .determine_missing_contracts()
            .await
            .unwrap()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn withdraw_coll<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        _oracle: &ContractInstance<Oracle<T>>,
        _pyth: &PythCore<T>,
        _redstone: &RedstoneCore<T>,
        mock_token: &Token<T>,
        _sorted_troves: &ContractInstance<SortedTroves<T>>,
        _trove_manager: &ContractInstance<TroveManagerContract<T>>,
        _active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let mock_asset_id: AssetId = mock_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .contract
            .methods()
            .withdraw_coll(amount, lower_hint, upper_hint, mock_asset_id.into())
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await
    }

    pub async fn withdraw_usdm<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        _oracle: &ContractInstance<Oracle<T>>,
        _pyth: &PythCore<T>,
        _redstone: &RedstoneCore<T>,
        mock_token: &Token<T>,
        _usdm_token: &ContractInstance<USDMToken<T>>,
        _fpt_staking: &ContractInstance<FPTStaking<T>>,
        _sorted_troves: &ContractInstance<SortedTroves<T>>,
        _trove_manager: &ContractInstance<TroveManagerContract<T>>,
        _active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let mock_asset_id: AssetId = mock_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .contract
            .methods()
            .withdraw_usdm(amount, lower_hint, upper_hint, mock_asset_id.into())
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await
    }

    pub async fn repay_usdm<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        _oracle: &ContractInstance<Oracle<T>>,
        _pyth: &PythCore<T>,
        _redstone: &RedstoneCore<T>,
        mock_token: &Token<T>,
        usdm_token: &ContractInstance<USDMToken<T>>,
        _sorted_troves: &ContractInstance<SortedTroves<T>>,
        _trove_manager: &ContractInstance<TroveManagerContract<T>>,
        _active_pool: &ContractInstance<ActivePool<T>>,
        _default_pool: &ContractInstance<DefaultPool<T>>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);
        let usdm_asset_id = usdm_token
            .contract
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdm_asset_id);

        let mock_asset_id: AssetId = mock_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .contract
            .methods()
            .repay_usdm(lower_hint, upper_hint, mock_asset_id.into())
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .determine_missing_contracts()
            .await
            .unwrap()
            .call()
            .await
    }

    pub async fn close_trove<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        oracle: &ContractInstance<Oracle<T>>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        mock_token: &Token<T>,
        usdm_token: &ContractInstance<USDMToken<T>>,
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);
        let usdm_asset_id: AssetId = usdm_token
            .contract
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        println!("usdm_asset_id: {:?}", usdm_asset_id);

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdm_asset_id);

        let mock_asset_id: AssetId = mock_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .contract
            .methods()
            .close_trove(mock_asset_id.into())
            .with_contracts(&[
                &oracle.contract,
                pyth,
                redstone,
                mock_token,
                &sorted_troves.contract,
                &trove_manager.contract,
                &active_pool.contract,
                &usdm_token.contract,
                &fpt_staking.contract,
            ])
            .with_contract_ids(&[
                borrow_operations.contract.contract_id().into(),
                borrow_operations.implementation_id.into(),
                sorted_troves.implementation_id.into(),
                sorted_troves.contract.contract_id().into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
                oracle.contract.contract_id().into(),
                oracle.implementation_id.into(),
                pyth.contract_id().into(),
                redstone.contract_id().into(),
                mock_token.contract_id().into(),
                usdm_token.contract.contract_id().into(),
                usdm_token.implementation_id.into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
    }

    pub async fn add_asset<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        oracle: ContractId,
        trove_manager: ContractId,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        return borrow_operations
            .contract
            .methods()
            .add_asset(asset.into(), trove_manager, oracle)
            .with_tx_policies(tx_params)
            .call()
            .await;
    }

    pub async fn set_pause_status<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        is_paused: bool,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .set_pause_status(is_paused)
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn get_pauser<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
    ) -> Result<CallResponse<Identity>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .get_pauser()
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn get_is_paused<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
    ) -> Result<CallResponse<bool>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .get_is_paused()
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn claim_coll<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        active_pool: &ContractInstance<ActivePool<T>>,
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        asset: AssetId,
    ) -> CallResponse<()> {
        borrow_operations
            .contract
            .methods()
            .claim_collateral(asset.into())
            .with_contracts(&[&active_pool.contract, &coll_surplus_pool.contract])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_contract_ids(&[
                borrow_operations.contract.contract_id().into(),
                borrow_operations.implementation_id.into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    // Add these new functions to the module
    pub async fn set_pauser<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        pauser: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .set_pauser(pauser)
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn transfer_owner<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        new_owner: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .transfer_owner(new_owner)
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn renounce_owner<T: Account + Clone>(
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        borrow_operations
            .contract
            .methods()
            .renounce_owner()
            .with_contract_ids(&[borrow_operations.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await
    }
}

pub mod borrow_operations_utils {
    use fuels::accounts::ViewOnlyAccount;
    use fuels::prelude::{Account, Wallet};
    use fuels::types::{Address, Identity};

    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::usdm_token::USDMToken;
    use crate::{data_structures::AssetContracts, interfaces::token::token_abi};

    pub async fn mint_token_and_open_trove<T: Account + Clone>(
        wallet: Wallet,
        asset_contracts: &AssetContracts<Wallet>,
        borrow_operations: &ContractInstance<BorrowOperations<T>>,
        usdm: &ContractInstance<USDMToken<Wallet>>,
        fpt_staking: &ContractInstance<FPTStaking<Wallet>>,
        active_pool: &ContractInstance<ActivePool<Wallet>>,
        sorted_troves: &ContractInstance<SortedTroves<Wallet>>,
        amount: u64,
        usdm_amount: u64,
    ) {
        token_abi::mint_to_id(
            &asset_contracts.asset,
            amount,
            Identity::Address(wallet.address().into()),
        )
        .await;

        let borrow_operations_healthy_wallet1 = ContractInstance::new(
            BorrowOperations::new(
                borrow_operations.contract.contract_id().clone(),
                wallet.clone(),
            ),
            borrow_operations.implementation_id.into(),
        );

        borrow_operations_abi::open_trove(
            &borrow_operations_healthy_wallet1,
            &asset_contracts.oracle,
            &asset_contracts.mock_pyth_oracle,
            &asset_contracts.mock_redstone_oracle,
            &asset_contracts.asset,
            &usdm,
            fpt_staking,
            &sorted_troves,
            &asset_contracts.trove_manager,
            &active_pool,
            amount,
            usdm_amount,
            Identity::Address(Address::zeroed()),
            Identity::Address(Address::zeroed()),
        )
        .await
        .unwrap();
    }
}
