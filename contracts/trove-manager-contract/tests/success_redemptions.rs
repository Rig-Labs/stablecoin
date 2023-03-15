use fuels::{prelude::AssetId, types::Identity};
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils, TroveManagerContract},
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_redemption_from_partially_closed() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    let balance = 10_000_000_000;

    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(healthy_wallet2.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(healthy_wallet3.address().into()),
    )
    .await;

    let borrow_operations_healthy_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        10_000_000_000,
        5_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let borrow_operations_healthy_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet2.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        9_000_000_000,
        5_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let borrow_operations_healthy_wallet3 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet3.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet3,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        8_000_000_000,
        5_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.oracle, 1_000_000).await;

    let redemption_amount: u64 = 3_000_000_000;

    let trove_manager_health1 = TroveManagerContract::new(
        contracts.trove_manager.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    trove_manager_abi::redeem_collateral(
        &trove_manager_health1,
        redemption_amount,
        10,
        0,
        0,
        None,
        None,
        &contracts.usdf,
        &contracts.fuel,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.coll_surplus_pool,
        &contracts.oracle,
        &contracts.default_pool,
    )
    .await;

    let active_pool_asset = active_pool_abi::get_asset(&contracts.active_pool)
        .await
        .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(&contracts.active_pool)
        .await
        .value;

    assert_eq!(active_pool_asset, 24_000_000_000);
    assert_eq!(active_pool_debt, 12_000_000_000);

    let provider = healthy_wallet1.get_provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.fuel.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(healthy_wallet1.address(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(fuel_balance, 3_000_000_000);

    trove_manager_utils::assert_trove_coll(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        5_000_000_000,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        2_000_000_000,
    )
    .await;
}
