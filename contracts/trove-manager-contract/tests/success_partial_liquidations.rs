use fuels::prelude::*;
use fuels::types::Identity;
use test_utils::{
    data_structures::{POST_LIQUIDATION_COLLATERAL_RATIO, PRECISION},
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        coll_surplus_pool::coll_surplus_pool_abi,
        default_pool::default_pool_abi,
        oracle::oracle_abi,
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed, pyth_price_feed_with_time, PYTH_PRECISION,
            PYTH_TIMESTAMP,
        },
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils, Status},
    },
    setup::common::setup_protocol,
    utils::{assert_within_threshold, calculate_cr, with_min_borrow_fee},
};

#[tokio::test]
async fn proper_partial_liquidation_enough_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 25_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
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
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        12_000 * PRECISION,
        10_100 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        20_000 * PRECISION,
        15_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let stability_pool_wallet2 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        wallet2.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        15_000 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed_with_time(1, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
    )
    .await;

    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it
    let res = trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(wallet1.address().into()),
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let logs = res.decode_logs();
    let liquidation_event = logs
        .results
        .iter()
        .find(|log| {
            log.as_ref()
                .unwrap()
                .contains("TrovePartialLiquidationEvent")
        })
        .expect("TrovePartialLiquidationEvent not found")
        .as_ref()
        .unwrap();

    assert!(
        liquidation_event.contains(&wallet1.address().hash().to_string()),
        "TrovePartialLiquidationEvent should contain user address"
    );

    let status = trove_manager_abi::get_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::Active);

    let coll = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let collateral_ratio = (coll as u128 * PRECISION as u128 / debt as u128) as u64;

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);
    assert_eq!(collateral_ratio, POST_LIQUIDATION_COLLATERAL_RATIO);

    let pending_asset_rewards = trove_manager_abi::get_pending_asset_reward(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    let pending_usdf_rewards = trove_manager_abi::get_pending_usdf_reward(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(wallet1.address().into()),
    )
    .await
    .value;

    assert_eq!(pending_asset_rewards, 0);
    assert_eq!(pending_usdf_rewards, 0);

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(wallet1.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}

#[tokio::test]
async fn proper_partial_liquidation_partial_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

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

    let starting_col: u64 = 12_000 * PRECISION;
    let starting_debt: u64 = 10_000 * PRECISION;

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        starting_col,
        starting_debt,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        10_000 * PRECISION,
        5_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        30_000 * PRECISION,
        15_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed_with_time(1, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
    )
    .await;

    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    let _response = trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::Active,
    )
    .await;

    let remaining_coll = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let remaining_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let collateral_ratio = calculate_cr(PRECISION, remaining_coll, remaining_debt);

    assert_eq!(collateral_ratio, POST_LIQUIDATION_COLLATERAL_RATIO);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .unwrap()
    .value;

    let gas_coll_compensation = 550 * PRECISION / 200;
    let expected_asset_in_sp = 550 * PRECISION - gas_coll_compensation;

    assert_within_threshold(
        asset,
        expected_asset_in_sp,
        &format!(
            "Asset in stability pool was not {}, but {}",
            asset, expected_asset_in_sp
        ),
    );

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION + remaining_coll);
    assert_eq!(
        active_pool_debt,
        with_min_borrow_fee(20_000 * PRECISION) + remaining_debt
    );

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    // 1.05 * 500_000_000
    let liquidated_amount = starting_col - remaining_coll;
    let gas_coll_compensation = liquidated_amount / 200;

    assert_within_threshold(
        default_pool_asset,
        liquidated_amount - gas_coll_compensation - expected_asset_in_sp,
        &format!(
            "Default pool asset was not {} but {}",
            liquidated_amount - gas_coll_compensation - expected_asset_in_sp,
            default_pool_asset,
        ),
    );

    assert_eq!(
        default_pool_debt,
        with_min_borrow_fee(starting_debt) - remaining_debt - 500 * PRECISION
    );

    let walet2_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(30_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        walet2_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet2_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(30_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        wallet2_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(10_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(10_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_asset_rewards = u128::from(default_pool_asset)
        * u128::from(remaining_coll)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_asset_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_usdf_rewards = u128::from(default_pool_debt) * u128::from(remaining_coll)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}

#[tokio::test]
async fn proper_partial_liquidation_empty_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

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

    let starting_col: u64 = 12_000 * PRECISION;
    let starting_debt: u64 = 10_000 * PRECISION;

    borrow_operations_abi::open_trove(
        &borrow_operations_liquidated_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        starting_col,
        starting_debt,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Open 2nd trove to deposit into stability pool
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        10_000 * PRECISION,
        5_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Open 3rd trove to deposit into stability pool with 3x the size of the 2nd trove
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        30_000 * PRECISION,
        15_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed_with_time(1, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
    )
    .await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::Active,
    )
    .await;

    let remaining_coll = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let remaining_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    let collateral_ratio = calculate_cr(PRECISION, remaining_coll, remaining_debt);

    assert_eq!(collateral_ratio, POST_LIQUIDATION_COLLATERAL_RATIO);

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(deposits, 0);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .unwrap()
    .value;

    assert_eq!(asset, 0);

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION + remaining_coll);
    assert_eq!(
        active_pool_debt,
        with_min_borrow_fee(20_000 * PRECISION) + remaining_debt
    );

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let liquidated_amount = starting_col - remaining_coll;
    let gas_coll_compensation = liquidated_amount / 200;
    // 1.05 * 500_000_000
    assert_eq!(
        default_pool_asset,
        liquidated_amount - gas_coll_compensation
    );
    assert_eq!(
        default_pool_debt,
        with_min_borrow_fee(starting_debt) - remaining_debt
    );

    let walet2_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(30_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        walet2_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet2_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(30_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        wallet2_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_asset_rewards: u128 = u128::from(default_pool_asset)
        * u128::from(10_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_asset_rewards.try_into().unwrap(),
    )
    .await;

    let wallet1_expected_usdf_rewards: u128 = u128::from(default_pool_debt)
        * u128::from(10_000 * PRECISION as u128)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        wallet1_expected_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_asset_rewards = u128::from(default_pool_asset)
        * u128::from(remaining_coll)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_asset_rewards.try_into().unwrap(),
    )
    .await;

    let liqudated_wallet_usdf_rewards = u128::from(default_pool_debt) * u128::from(remaining_coll)
        / u128::from(40_000 * PRECISION + remaining_coll);

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        liqudated_wallet_usdf_rewards.try_into().unwrap(),
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}
