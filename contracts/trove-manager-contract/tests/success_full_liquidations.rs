use fuels::types::Identity;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        coll_surplus_pool::coll_surplus_pool_abi,
        default_pool::default_pool_abi,
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils, Status},
    },
    setup::common::setup_protocol,
    utils::{with_liquidation_penalty, with_min_borrow_fee},
};

#[tokio::test]
async fn proper_full_liquidation_enough_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();

    let balance = 25_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
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

    let usdf_deposit_to_be_liquidated = 1_000 * PRECISION;
    let asset_deposit_to_be_liquidated = 1_100 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        asset_deposit_to_be_liquidated,
        usdf_deposit_to_be_liquidated,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        10_000 * PRECISION,
        5_000 * PRECISION,
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
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].active_pool,
        &contracts.asset_contracts[0].default_pool,
        &contracts.asset_contracts[0].coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let status = trove_manager_abi::get_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::ClosedByLiquidation);

    let coll = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    assert_eq!(coll, 0);

    let debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    assert_eq!(debt, 0);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    let liquidated_net_debt = with_min_borrow_fee(usdf_deposit_to_be_liquidated);
    assert_eq!(deposits, 5_000 * PRECISION - liquidated_net_debt);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    // 5% Penalty on 1_000* PRECISION of debt
    let asset_with_min_borrow_fee = with_min_borrow_fee(1_050 * PRECISION);
    assert_eq!(asset, asset_with_min_borrow_fee);

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    assert_eq!(active_pool_asset, 10_000 * PRECISION);

    let active_pool_debt_with_min_borrow_fee = with_min_borrow_fee(5_000 * PRECISION);
    assert_eq!(active_pool_debt, active_pool_debt_with_min_borrow_fee);

    let default_pool_asset =
        default_pool_abi::get_asset(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    let default_pool_debt =
        default_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.asset_contracts[0].coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    // Prices are the same
    assert_eq!(
        liq_coll_surplus,
        asset_deposit_to_be_liquidated - with_liquidation_penalty(liquidated_net_debt),
        "Liquidated wallet collateral surplus was not 50_000"
    );
}

#[tokio::test]
async fn proper_full_liquidation_partial_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
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

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_100 * PRECISION,
        1_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        10_000 * PRECISION,
        5_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        30_000 * PRECISION,
        15_000 * PRECISION,
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
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        500 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].active_pool,
        &contracts.asset_contracts[0].default_pool,
        &contracts.asset_contracts[0].coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    // 5% Penalty on 1_000* PRECISION of debt
    assert_eq!(
        asset,
        525 * PRECISION,
        "Incorrect asset amount in stability pool"
    );

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION);
    assert_eq!(active_pool_debt, with_min_borrow_fee(20_000 * PRECISION));

    let default_pool_asset =
        default_pool_abi::get_asset(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    let default_pool_debt =
        default_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    // 1.05 * 500_000_000
    let debt_being_redistributed = with_min_borrow_fee(1_000 * PRECISION) - 500 * PRECISION;
    let asset_being_redistributed = with_liquidation_penalty(debt_being_redistributed);
    assert_eq!(
        default_pool_asset, asset_being_redistributed,
        "Incorrect asset amount in default pool"
    );
    assert_eq!(default_pool_debt, debt_being_redistributed);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        asset_being_redistributed / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        debt_being_redistributed / 4,
    )
    .await;

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        asset_being_redistributed * 3 / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        debt_being_redistributed * 3 / 4,
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.asset_contracts[0].coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    assert_eq!(
        liq_coll_surplus,
        1_100 * PRECISION - with_liquidation_penalty(with_min_borrow_fee(1_000 * PRECISION)),
        "Liquidated wallet collateral surplus was not 50_000_000"
    );
}

#[tokio::test]
async fn proper_full_liquidation_empty_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
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

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_100 * PRECISION,
        1_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        10_000 * PRECISION,
        5_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        30_000 * PRECISION,
        15_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    let _response = trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].active_pool,
        &contracts.asset_contracts[0].default_pool,
        &contracts.asset_contracts[0].coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(asset, 0);

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION);
    assert_eq!(active_pool_debt, with_min_borrow_fee(20_000 * PRECISION));

    let default_pool_asset =
        default_pool_abi::get_asset(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    let default_pool_debt =
        default_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    // 1.05 * 500_000_000
    let expected_default_pool_asset =
        with_liquidation_penalty(with_min_borrow_fee(1_000 * PRECISION));
    let expected_default_pool_debt = with_min_borrow_fee(1_000 * PRECISION);
    assert_eq!(default_pool_asset, expected_default_pool_asset);
    assert_eq!(default_pool_debt, expected_default_pool_debt);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        expected_default_pool_asset / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        expected_default_pool_debt / 4,
    )
    .await;

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        expected_default_pool_asset * 3 / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        expected_default_pool_debt * 3 / 4,
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.asset_contracts[0].coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    assert_eq!(
        liq_coll_surplus,
        1_100 * PRECISION - expected_default_pool_asset,
        "Liquidated wallet collateral surplus was not 50_000"
    );
}
