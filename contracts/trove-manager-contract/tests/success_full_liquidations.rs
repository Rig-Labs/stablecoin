use std::cmp::min;

use fuels::types::{AssetId, Identity};
use test_utils::{
    data_structures::PRECISION,
    deploy::deployment::assert_within_threshold,
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

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();

    let balance = 25_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].asset,
        5_000 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.aswith_contracts[0].oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let status = trove_manager_abi::get_trove_status(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(status, Status::ClosedByLiquidation);

    let coll = trove_manager_abi::get_trove_coll(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    assert_eq!(coll, 0);

    let debt = trove_manager_abi::get_trove_debt(
        &contracts.aswith_contracts[0].trove_manager,
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
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    // 10% Penalty on 1_000* PRECISION of debt
    let mut liquidated_asset_amount_transfered_to_sp = with_min_borrow_fee(1_100 * PRECISION);
    liquidated_asset_amount_transfered_to_sp = min(
        liquidated_asset_amount_transfered_to_sp,
        asset_deposit_to_be_liquidated,
    );

    let coll_gas_fee = liquidated_asset_amount_transfered_to_sp / 200;
    liquidated_asset_amount_transfered_to_sp -= coll_gas_fee;

    assert_eq!(asset, liquidated_asset_amount_transfered_to_sp);

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(active_pool_asset, 10_000 * PRECISION);

    let active_pool_debt_with_min_borrow_fee = with_min_borrow_fee(5_000 * PRECISION);
    assert_eq!(active_pool_debt, active_pool_debt_with_min_borrow_fee);

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        &contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    // Prices are the same
    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}

#[tokio::test]
async fn proper_full_liquidation_partial_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].asset,
        500 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::liquidate(
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.aswith_contracts[0].oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.aswith_contracts[0].trove_manager,
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
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    // 10% Penalty on 1_000* PRECISION of debt
    let mut expected_asset_in_sp = 1_100 * PRECISION * 500 / 1005;
    expected_asset_in_sp -= expected_asset_in_sp / 200;

    assert_within_threshold(
        asset,
        expected_asset_in_sp,
        "Incorrect asset amount in stability pool",
    );

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION);
    assert_eq!(active_pool_debt, with_min_borrow_fee(20_000 * PRECISION));

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let asset_amount_to_sp = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;
    println!("asset_amount_to_sp: {}", asset_amount_to_sp);

    // 1.10 * 500_000_000
    let debt_being_redistributed = with_min_borrow_fee(1_000 * PRECISION) - 500 * PRECISION;
    let mut asset_being_redistributed = 1_100 * PRECISION - asset_amount_to_sp;

    let coll_gas_fee = 1_100 * PRECISION / 200;
    asset_being_redistributed -= coll_gas_fee;

    assert_eq!(
        default_pool_asset, asset_being_redistributed,
        "Incorrect asset amount in default pool"
    );
    assert_eq!(default_pool_debt, debt_being_redistributed);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        asset_being_redistributed / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        debt_being_redistributed / 4,
    )
    .await;

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        asset_being_redistributed * 3 / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        debt_being_redistributed * 3 / 4,
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        &contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}

#[tokio::test]
async fn proper_full_liquidation_empty_sp() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();

    let balance = 35_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
        balance,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.aswith_contracts[0].asset,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
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
        &contracts.aswith_contracts[0].oracle,
        &contracts.aswith_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.active_pool,
        30_000 * PRECISION,
        15_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 1 * PRECISION).await;
    // Wallet 1 has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    let _response = trove_manager_abi::liquidate(
        &contracts.aswith_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.aswith_contracts[0].oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.aswith_contracts[0].trove_manager,
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
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(asset, 0);

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(active_pool_asset, 40_000 * PRECISION);
    assert_eq!(active_pool_debt, with_min_borrow_fee(20_000 * PRECISION));

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    // 1.10 * 500_000_000
    let mut expected_default_pool_asset =
        with_liquidation_penalty(with_min_borrow_fee(1_000 * PRECISION));
    // max available to liquidate is 1_100 * PRECISION
    expected_default_pool_asset = min(expected_default_pool_asset, 1_100 * PRECISION);
    let expected_default_pool_debt = with_min_borrow_fee(1_000 * PRECISION);
    let gas_compensation = expected_default_pool_asset / 200;
    expected_default_pool_asset -= gas_compensation;

    assert_eq!(default_pool_asset, expected_default_pool_asset);
    assert_eq!(default_pool_debt, expected_default_pool_debt);

    // Check that the admin got the gas compensation
    let provider = admin.provider().unwrap();
    let asset_id = AssetId::from(*contracts.aswith_contracts[0].asset.contract_id().hash());

    let asset_balance = provider
        .get_asset_balance(admin.address(), asset_id)
        .await
        .unwrap();
    assert_eq!(asset_balance, gas_compensation);

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        expected_default_pool_asset / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet1.address().into()),
        expected_default_pool_debt / 4,
    )
    .await;

    trove_manager_utils::assert_pending_asset_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        expected_default_pool_asset * 3 / 4,
    )
    .await;

    trove_manager_utils::assert_pending_usdf_rewards(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        expected_default_pool_debt * 3 / 4,
    )
    .await;

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        &contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    assert_eq!(
        liq_coll_surplus,
        1_100 * PRECISION - expected_default_pool_asset - gas_compensation,
        "Liquidated wallet collateral surplus was not 50_000"
    );
}
