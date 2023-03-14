use fuels::types::Identity;
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        default_pool::default_pool_abi,
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils, Status},
    },
    setup::common::{print_response, setup_protocol},
};

#[tokio::test]
async fn proper_partial_liquidation_enough_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 25_000_000_000;
    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(wallet2.address().into()),
    )
    .await;

    let borrow_operations_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet1.clone(),
    );

    let borrow_operations_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet2.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
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
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        20_000_000_000,
        15_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let stability_pool_wallet2 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        wallet2.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &contracts.usdf,
        15_000_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.oracle, 1_000_000).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.trove_manager,
        &contracts.stability_pool,
        &contracts.oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap();

    let status = trove_manager_abi::get_trove_status(
        &contracts.trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::Active);

    let coll = trove_manager_abi::get_trove_coll(
        &contracts.trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let debt = trove_manager_abi::get_trove_debt(
        &contracts.trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let collateral_ratio = coll * 1_000_000 / debt;

    let default_pool_asset = default_pool_abi::get_asset(&contracts.default_pool)
        .await
        .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(&contracts.default_pool)
        .await
        .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);
    assert_eq!(collateral_ratio, 1_300_000);

    let pending_asset_rewards = trove_manager_abi::get_pending_asset_reward(
        &contracts.trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let pending_usdf_rewards = trove_manager_abi::get_pending_usdf_reward(
        &contracts.trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    assert_eq!(pending_asset_rewards, 0);
    assert_eq!(pending_usdf_rewards, 0);
}

#[tokio::test]
async fn proper_partial_liquidation_partial_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000_000_000;
    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

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

    let borrow_operations_liquidated_wallet = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    let borrow_operations_healthy_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    let borrow_operations_healthy_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet2.clone(),
    );

    let starting_col: u64 = 11_000_000_000;
    let starting_debt: u64 = 10_000_000_000;

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        starting_col,
        starting_debt,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
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

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        30_000_000_000,
        15_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let stability_pool_healthy_wallet1 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_healthy_wallet1,
        &contracts.usdf,
        500_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.oracle, 1_000_000).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    let response = trove_manager_abi::liquidate(
        &contracts.trove_manager,
        &contracts.stability_pool,
        &contracts.oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .unwrap();

    print_response(response);

    trove_manager_utils::assert_trove_status(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::Active,
    )
    .await;

    let remaining_coll = trove_manager_abi::get_trove_coll(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let remaining_debt = trove_manager_abi::get_trove_debt(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let collateral_ratio = remaining_coll * 1_000_000 / remaining_debt;

    assert_eq!(collateral_ratio, 1_300_000);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset, 525_000_000);

    let active_pool_asset = active_pool_abi::get_asset(&contracts.active_pool)
        .await
        .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(&contracts.active_pool)
        .await
        .value;

    assert_eq!(active_pool_asset, 40_000_000_000 + remaining_coll);
    assert_eq!(active_pool_debt, 20_000_000_000 + remaining_debt);

    let default_pool_asset = default_pool_abi::get_asset(&contracts.default_pool)
        .await
        .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(&contracts.default_pool)
        .await
        .value;

    // 1.05 * 500_000_000
    assert_eq!(
        default_pool_asset,
        starting_col - remaining_coll - 525_000_000
    );
    assert_eq!(
        default_pool_debt,
        starting_debt - remaining_debt - 500_000_000
    );

    let walet2_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(30_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        walet2_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet2_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(30_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        wallet2_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(10_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(10_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_asset_rewards = u128::from(default_pool_asset)
        * u128::from(remaining_coll)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_asset_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_usdf_rewards = u128::from(default_pool_debt) * u128::from(remaining_coll)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_usdf_rewards.try_into().unwrap(),
    )
    .await;
}

#[tokio::test]
async fn proper_full_liquidation_empty_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5).await;

    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000_000_000;
    token_abi::mint_to_id(
        &contracts.fuel,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

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

    let borrow_operations_liquidated_wallet = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    let borrow_operations_healthy_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    let borrow_operations_healthy_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet2.clone(),
    );

    let starting_col: u64 = 11_000_000_000;
    let starting_debt: u64 = 10_000_000_000;

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        starting_col,
        starting_debt,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
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

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        30_000_000_000,
        15_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.oracle, 1_000_000).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    let response = trove_manager_abi::liquidate(
        &contracts.trove_manager,
        &contracts.stability_pool,
        &contracts.oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .unwrap();

    print_response(response);

    trove_manager_utils::assert_trove_status(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::Active,
    )
    .await;

    let remaining_coll = trove_manager_abi::get_trove_coll(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let remaining_debt = trove_manager_abi::get_trove_debt(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let collateral_ratio = remaining_coll * 1_000_000 / remaining_debt;

    assert_eq!(collateral_ratio, 1_300_000);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset, 0);

    let active_pool_asset = active_pool_abi::get_asset(&contracts.active_pool)
        .await
        .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(&contracts.active_pool)
        .await
        .value;

    assert_eq!(active_pool_asset, 40_000_000_000 + remaining_coll);
    assert_eq!(active_pool_debt, 20_000_000_000 + remaining_debt);

    let default_pool_asset = default_pool_abi::get_asset(&contracts.default_pool)
        .await
        .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(&contracts.default_pool)
        .await
        .value;

    // 1.05 * 500_000_000
    assert_eq!(default_pool_asset, starting_col - remaining_coll);
    assert_eq!(default_pool_debt, starting_debt - remaining_debt);

    let walet2_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(30_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        walet2_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet2_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(30_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        wallet2_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(10_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(10_000_000_000 as u128)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_asset_rewards = u128::from(default_pool_asset)
        * u128::from(remaining_coll)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_asset_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_usdf_rewards = u128::from(default_pool_debt) * u128::from(remaining_coll)
        / u128::from(40_000_000_000 + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_usdf_rewards.try_into().unwrap(),
    )
    .await;
}
