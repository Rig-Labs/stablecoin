use fuels::types::Identity;
use test_utils::{
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        token::token_abi,
        trove_manager::trove_manager_abi,
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn fails_to_liquidate_trove_not_under_mcr() {
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        _admin,
        mut wallets,
        stability_pool,
        default_pool,
    ) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&oracle, 10_000_000).await;

    let wallet1 = wallets.pop().unwrap();

    let balance = 25_000_000_000;
    token_abi::mint_to_id(
        &fuel_token,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    let borrow_operations_wallet1 =
        BorrowOperations::new(borrow_operations.contract_id().clone(), wallet1.clone());

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_100_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_abi::liquidate(
        &trove_manager,
        &stability_pool,
        &oracle,
        &sorted_troves,
        &active_pool,
        &default_pool,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .expect_err("Should fail not below MCR");
}
