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
    use fuels::prelude::{AssetId, CallParameters, Error};

    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::oracle::Oracle;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::token::Token;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use crate::interfaces::usdf_token::USDFToken;

    pub async fn initialize(
        borrow_operations: &BorrowOperations,
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

    pub async fn open_trove(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        usdf_token: &USDFToken,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
        fuel_amount_deposit: u64,
        usdf_amount_withdrawn: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);
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
            ])
            .append_variable_outputs(3)
            .tx_params(tx_params)
            .call()
            .await
    }

    pub async fn add_coll(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        usdf_token: &USDFToken,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
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

    pub async fn withdraw_coll(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
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

    pub async fn withdraw_usdf(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        usdf_token: &USDFToken,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
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
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn repay_usdf(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        usdf_token: &USDFToken,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
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

    pub async fn close_trove(
        borrow_operations: &BorrowOperations,
        oracle: &Oracle,
        fuel_token: &Token,
        usdf_token: &USDFToken,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        active_pool: &ActivePool,
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
            ])
            .append_variable_outputs(1)
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset(
        borrow_operations: &BorrowOperations,
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
