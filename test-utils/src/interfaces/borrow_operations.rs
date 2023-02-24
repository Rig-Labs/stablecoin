use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
};

abigen!(Contract(
    name = "BorrowOperations",
    abi = "contracts/borrow-operations-contract/out/debug/borrow-operations-contract-abi.json"
));

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
