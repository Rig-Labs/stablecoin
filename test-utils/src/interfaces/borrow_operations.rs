use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "BorrowOperations",
    abi = "contracts/borrow-operations-contract/out/debug/borrow-operations-contract-abi.json"
));

pub mod borrow_operations_abi {
    use fuels::prelude::{Account, AssetId, CallParameters, Error};

    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::oracle::Oracle;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::token::Token;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use crate::interfaces::usdf_token::USDFToken;

    pub async fn initialize<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        usdf_contract: ContractId,
        fpt_staking_contract: ContractId,
        stability_pool_contract: ContractId,
        protocol_manager_contract: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        borrow_operations
            .methods()
            .initialize(
                usdf_contract,
                fpt_staking_contract,
                stability_pool_contract,
                protocol_manager_contract,
            )
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn open_trove<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        fuel_amount_deposit: u64,
        usdf_amount_withdrawn: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default()
            .set_gas_price(1)
            .set_gas_limit(2000000);
        let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(fuel_amount_deposit)
            .set_asset_id(fuel_asset_id);

        borrow_operations
            .methods()
            .open_trove(
                usdf_amount_withdrawn,
                upper_hint,
                lower_hint,
                fuel_token.contract_id().into(),
            )
            .call_params(call_params)
            .unwrap()
            .set_contracts(&[
                oracle,
                active_pool,
                fuel_token,
                usdf_token,
                sorted_troves,
                trove_manager,
                fpt_staking
            ])
            .append_variable_outputs(3)
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn add_coll<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(fuel_asset_id);

        borrow_operations
            .methods()
            .add_coll(lower_hint, upper_hint, fuel_token.contract_id().into())
            .call_params(call_params)
            .unwrap()
            .set_contracts(&[
                oracle,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn withdraw_coll<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);

        borrow_operations
            .methods()
            .withdraw_coll(
                amount,
                lower_hint,
                upper_hint,
                fuel_token.contract_id().into(),
            )
            .set_contracts(&[
                oracle,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn withdraw_usdf<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        borrow_operations
            .methods()
            .withdraw_usdf(
                amount,
                lower_hint,
                upper_hint,
                fuel_token.contract_id().into(),
            )
            .set_contracts(&[
                oracle,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
                fpt_staking,
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn repay_usdf<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
        lower_hint: Identity,
        upper_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);
        let usdf_asset_id = AssetId::from(*usdf_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(usdf_asset_id);

        borrow_operations
            .methods()
            .repay_usdf(lower_hint, upper_hint, fuel_token.contract_id().into())
            .set_contracts(&[
                oracle,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
    }

    pub async fn close_trove<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        oracle: &Oracle<T>,
        fuel_token: &Token<T>,
        usdf_token: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        sorted_troves: &SortedTroves<T>,
        trove_manager: &TroveManagerContract<T>,
        active_pool: &ActivePool<T>,
        amount: u64,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);
        let usdf_asset_id = AssetId::from(*usdf_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(usdf_asset_id);

        borrow_operations
            .methods()
            .close_trove(fuel_token.contract_id().into())
            .set_contracts(&[
                oracle,
                fuel_token,
                sorted_troves,
                trove_manager,
                active_pool,
                usdf_token,
                fpt_staking,
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        borrow_operations: BorrowOperations<T>,
        oracle: ContractId,
        sorted_troves: ContractId,
        trove_manager: ContractId,
        active_pool: ContractId,
        asset: ContractId,
        coll_surplus_pool_contract: ContractId,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);

        borrow_operations
            .methods()
            .add_asset(
                asset,
                trove_manager,
                sorted_troves,
                oracle,
                active_pool,
                coll_surplus_pool_contract,
            )
            .tx_params(tx_params)
            .call()
            .await
    }
}

pub mod borrow_operations_utils {
    use fuels::prelude::{Account, WalletUnlocked};

    use super::*;
    use crate::interfaces::usdf_token::USDFToken;
    use crate::{interfaces::token::token_abi, setup::common::AssetContracts};
    use crate::interfaces::fpt_staking::FPTStaking;

    pub async fn mint_token_and_open_trove<T: Account>(
        wallet: WalletUnlocked,
        asset_contracts: &AssetContracts<WalletUnlocked>,
        borrow_operations: &BorrowOperations<T>,
        usdf: &USDFToken<WalletUnlocked>,
        fpt_staking: &FPTStaking<WalletUnlocked>,
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
            &asset_contracts.asset,
            &usdf,
            fpt_staking,
            &asset_contracts.sorted_troves,
            &asset_contracts.trove_manager,
            &asset_contracts.active_pool,
            amount,
            usdf_amount,
            Identity::Address([0; 32].into()),
            Identity::Address([0; 32].into()),
        )
        .await
        .unwrap();

    }
}
