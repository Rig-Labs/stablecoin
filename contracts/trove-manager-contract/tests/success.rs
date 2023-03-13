use fuels::types::Identity;
use test_utils::{
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, Status},
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_full_liquidation_enough_usdf_in_sp() {
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
        _,
    ) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&oracle, 10_000_000).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 25_000_000_000;
    token_abi::mint_to_id(
        &fuel_token,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &fuel_token,
        balance,
        Identity::Address(wallet2.address().into()),
    )
    .await;

    let borrow_operations_wallet1 =
        BorrowOperations::new(borrow_operations.contract_id().clone(), wallet1.clone());

    let borrow_operations_wallet2 =
        BorrowOperations::new(borrow_operations.contract_id().clone(), wallet2.clone());

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

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        10_000_000_000,
        5_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let stability_pool_wallet2 =
        StabilityPool::new(stability_pool.contract_id().clone(), wallet2.clone());

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &usdf_token,
        5_000_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&oracle, 1_000_000).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &trove_manager,
        &stability_pool,
        &oracle,
        &sorted_troves,
        &active_pool,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap();

    let status = trove_manager_abi::get_trove_status(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::ClosedByLiquidation);

    let coll = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    assert_eq!(coll, 0);

    let debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    assert_eq!(debt, 0);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 4_000_000_000);

    let asset = stability_pool_abi::get_asset(&stability_pool)
        .await
        .unwrap()
        .value;

    // 5% Penalty on 1_000_000_000 of debt
    assert_eq!(asset, 1_050_000_000);
}

#[tokio::test]
async fn proper_partial_liquidation_enough_usdf_in_sp() {
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
        _,
    ) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&oracle, 10_000_000).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 25_000_000_000;
    token_abi::mint_to_id(
        &fuel_token,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &fuel_token,
        balance,
        Identity::Address(wallet2.address().into()),
    )
    .await;

    let borrow_operations_wallet1 =
        BorrowOperations::new(borrow_operations.contract_id().clone(), wallet1.clone());

    let borrow_operations_wallet2 =
        BorrowOperations::new(borrow_operations.contract_id().clone(), wallet2.clone());

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        12_000_000_000,
        10_100_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        20_000_000_000,
        15_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let stability_pool_wallet2 =
        StabilityPool::new(stability_pool.contract_id().clone(), wallet2.clone());

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &usdf_token,
        15_000_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&oracle, 1_000_000).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &trove_manager,
        &stability_pool,
        &oracle,
        &sorted_troves,
        &active_pool,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap();

    let status = trove_manager_abi::get_trove_status(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::Active);

    let coll = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let collateral_ratio = coll * 1_000_000 / debt;

    println!("collateral ratio: {}", collateral_ratio);
    println!("collateral: {}", coll);
    println!("debt: {}", debt);
    assert_eq!(collateral_ratio, 1_300_000);
    // TODO Check rest of the values in other contracts
}
