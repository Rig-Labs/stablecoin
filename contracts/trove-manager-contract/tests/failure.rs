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
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let wallet1 = wallets.pop().unwrap();

    let balance = 25_000_000_000;
    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    let borrow_operations_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet1.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        1_100_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_abi::liquidate(
        &contracts.trove_manager,
        &contracts.stability_pool,
        &contracts.oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .expect_err("Improper liquidation of trove not below MCR");
}
