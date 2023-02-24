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
    use fuels::prelude::{AssetId, CallParameters};

    use super::*;
    use crate::interfaces::oracle::Oracle;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::token::Token;
    use crate::interfaces::trove_manager::TroveManagerContract;

    pub async fn initialize(
        borrow_operations: &BorrowOperations,
        trove_manager_contract: ContractId,
        sorted_troves_contract: ContractId,
        oracle_contract: ContractId,
        asset_contract: ContractId,
        usdf_contract: ContractId,
        fpt_staking_contract: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        borrow_operations
            .methods()
            .initialize(
                trove_manager_contract,
                sorted_troves_contract,
                oracle_contract,
                asset_contract,
                usdf_contract,
                fpt_staking_contract,
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
        usdf_token: &Token,
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
        max_fee_percentage: u64,
        fuel_amount_deposit: u64,
        usdf_amount_withdrawn: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));
        let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

        let call_params: CallParameters = CallParameters {
            amount: fuel_amount_deposit,
            asset_id: fuel_asset_id,
            gas_forwarded: None,
        };

        borrow_operations
            .methods()
            .open_trove(
                max_fee_percentage,
                usdf_amount_withdrawn,
                upper_hint,
                lower_hint,
            )
            .call_params(call_params)
            .set_contracts(&[oracle, fuel_token, usdf_token, sorted_troves, trove_manager])
            .append_variable_outputs(3)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }
}
