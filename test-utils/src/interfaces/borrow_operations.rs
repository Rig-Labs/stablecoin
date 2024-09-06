use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "BorrowOperations",
    abi = "contracts/borrow-operations-contract/out/debug/borrow-operations-contract-abi.json"
));

pub mod borrow_operations_abi {
    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::oracle::Oracle;
    use crate::interfaces::pyth_oracle::PythCore;
    use crate::interfaces::redstone_oracle::RedstoneCore;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::token::Token;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use crate::interfaces::usdf_token::USDFToken;
    use fuels::prelude::Account;
    use fuels::prelude::{CallParameters, ContractId, Error, TxPolicies};
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::types::{AssetId, Identity};

    pub async fn initialize<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        usdf_contract: ContractId,
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
            .methods()
            .initialize(
                usdf_contract,
                fpt_staking_contract,
                protocol_manager_contract,
                coll_surplus_pool_contract,
                active_pool_contract,
                sorted_troves_contract,
            )
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn open_trove<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        mock_pyth: &PythCore<T>,
        mock_redstone: &RedstoneCore<T>,
        asset_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        fuel_amount_deposit: u64,
        usdf_amount_withdrawn: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_witness_limit(2000000)
            .with_script_gas_limit(2000000);

        let asset_id = asset_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(fuel_amount_deposit)
            .with_asset_id(asset_id);
        println!("call_params: {:?}", call_params);

        return borrow_operations
            .methods()
            .open_trove(usdf_amount_withdrawn, upper_hint, lower_hint)
            .call_params(call_params)
            .unwrap()
            .with_contracts(&[
                oracle,
                mock_pyth,
                mock_redstone,
                active_pool,
                asset_token,
                usdf_token,
                sorted_troves,
                trove_manager,
                fpt_staking,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .with_tx_policies(tx_params)
            .call()
            .await;
    }

    pub async fn add_coll<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let fuel_asset_id: AssetId = fuel_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(fuel_asset_id);

        borrow_operations
            .methods()
            .add_coll(lower_hint, upper_hint)
            .call_params(call_params)
            .unwrap()
            .with_contracts(&[
                oracle,
                pyth,
                redstone,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn withdraw_coll<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        fuel_token: &Token<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let fuel_asset_id: AssetId = fuel_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .methods()
            .withdraw_coll(amount, lower_hint, upper_hint, fuel_asset_id.into())
            .with_contracts(&[
                oracle,
                pyth,
                redstone,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn withdraw_usdf<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let fuel_asset_id: AssetId = fuel_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .methods()
            .withdraw_usdf(amount, lower_hint, upper_hint, fuel_asset_id.into())
            .with_contracts(&[
                oracle,
                pyth,
                redstone,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
                fpt_staking,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn repay_usdf<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);
        let usdf_asset_id = usdf_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdf_asset_id);

        let fuel_asset_id: AssetId = fuel_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .methods()
            .repay_usdf(lower_hint, upper_hint, fuel_asset_id.into())
            .with_contracts(&[
                oracle,
                pyth,
                redstone,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
    }

    pub async fn close_trove<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        pyth: &PythCore<T>,
        redstone: &RedstoneCore<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);
        let usdf_asset_id: AssetId = usdf_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        println!("usdf_asset_id: {:?}", usdf_asset_id);

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdf_asset_id);

        let fuel_asset_id: AssetId = fuel_token
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        borrow_operations
            .methods()
            .close_trove(fuel_asset_id.into())
            .with_contracts(&[
                oracle,
                pyth,
                redstone,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
                fpt_staking,
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(3))
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        borrow_operations: BorrowOperations<T>,
        oracle: ContractId,
        trove_manager: ContractId,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        return borrow_operations
            .methods()
            .add_asset(asset.into(), trove_manager, oracle)
            .with_tx_policies(tx_params)
            .call()
            .await;
    }
}

pub mod borrow_operations_utils {
    use fuels::prelude::{Account, WalletUnlocked};
    use fuels::types::{Address, Identity};

    use super::*;
    use crate::interfaces::active_pool;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::usdf_token::USDFToken;
    use crate::{interfaces::token::token_abi, setup::common::AssetContracts};

    pub async fn mint_token_and_open_trove<T: Account>(
        wallet: WalletUnlocked,
        asset_contracts: &AssetContracts<WalletUnlocked>,
        borrow_operations: &BorrowOperations<T>,
        usdf: &USDFToken<WalletUnlocked>,
        fpt_staking: &FPTStaking<WalletUnlocked>,
        active_pool: &active_pool::ActivePool<WalletUnlocked>,
        sorted_troves: &SortedTroves<WalletUnlocked>,
        amount: u64,
        usdf_amount: u64,
    ) {
        token_abi::mint_to_id(
            &asset_contracts.asset,
            amount,
            Identity::Address(wallet.address().into()),
        )
        .await;

        let borrow_operations_healthy_wallet1 =
            BorrowOperations::new(borrow_operations.contract_id().clone(), wallet.clone());

        borrow_operations_abi::open_trove(
            &borrow_operations_healthy_wallet1,
            &asset_contracts.oracle,
            &asset_contracts.mock_pyth_oracle,
            &asset_contracts.mock_redstone_oracle,
            &asset_contracts.asset,
            &usdf,
            fpt_staking,
            &sorted_troves,
            &asset_contracts.trove_manager,
            &active_pool,
            amount,
            usdf_amount,
            Identity::Address(Address::zeroed()),
            Identity::Address(Address::zeroed()),
        )
        .await
        .unwrap();
    }
}
