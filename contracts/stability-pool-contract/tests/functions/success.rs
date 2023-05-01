use crate::utils::setup::setup;
use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, stability_pool_utils, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils},
    },
    setup::common::{assert_within_threshold, print_response, setup_protocol},
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn proper_initialization() {
    let (stability_pool, _, fuel, _, _, _) = setup(Some(4)).await;

    stability_pool_utils::assert_pool_asset(&stability_pool, 0, fuel.contract_id().into()).await;

    stability_pool_utils::assert_total_usdf_deposits(&stability_pool, 0).await;
}

#[tokio::test]
async fn proper_stability_deposit() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let res = stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        600 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_pool_asset(
        &contracts.stability_pool,
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_total_usdf_deposits(&contracts.stability_pool, 600 * PRECISION)
        .await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        600 * PRECISION,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;
}

#[tokio::test]
async fn proper_stability_widthdrawl() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        600 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_abi::withdraw_from_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_pool_asset(
        &contracts.stability_pool,
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_total_usdf_deposits(&contracts.stability_pool, 300 * PRECISION)
        .await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        300 * PRECISION,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;
}

#[tokio::test]
async fn proper_one_sp_depositor_position() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;
    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    let init_stability_deposit = 1_500 * PRECISION;
    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        init_stability_deposit,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
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

    // Since the entire debt is liquidated including the borrow fee,
    // the asset recieved includes the 0.5% fee
    let asset_with_fee_adjustment = with_min_borrow_fee(1_050 * PRECISION);
    let debt_with_fee_adjustment = with_min_borrow_fee(1_000 * PRECISION);

    stability_pool_utils::assert_pool_asset(
        &contracts.stability_pool,
        asset_with_fee_adjustment,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_total_usdf_deposits(
        &contracts.stability_pool,
        init_stability_deposit - debt_with_fee_adjustment,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        asset_with_fee_adjustment,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    // 500 - 0.5% fee
    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        init_stability_deposit - debt_with_fee_adjustment,
    )
    .await;

    // Makes a 2nd deposit to the Stability Pool
    let second_deposit = 1_000 * PRECISION;

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        second_deposit,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        init_stability_deposit - debt_with_fee_adjustment + second_deposit,
    )
    .await;

    // Gain has been withdrawn and resset
    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    let provider = admin.provider().unwrap();

    let fuel_asset_id: AssetId =
        AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(admin.address(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(fuel_balance, asset_with_fee_adjustment);
}

#[tokio::test]
async fn proper_many_depositors_distribution() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;
    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let depositor_2 = wallets.pop().unwrap();
    let depositor_3 = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        2_000 * PRECISION,
    )
    .await
    .unwrap();

    let usdf_asset_id: AssetId = AssetId::from(*contracts.usdf.contract_id().hash());
    let tx_params = TxParameters::default().set_gas_price(1);

    admin
        .transfer(
            depositor_2.address().into(),
            500 * PRECISION,
            usdf_asset_id,
            tx_params,
        )
        .await
        .unwrap();

    admin
        .transfer(
            depositor_3.address().into(),
            500 * PRECISION,
            usdf_asset_id,
            tx_params,
        )
        .await
        .unwrap();

    let depositor_2_sp = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        depositor_2.clone(),
    );

    let depositor_3_sp = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        depositor_3.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &depositor_2_sp,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        500 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &depositor_3_sp,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        500 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
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

    let asset_with_fee_adjustment = with_min_borrow_fee(1_050 * PRECISION);
    let debt_paid_off = with_min_borrow_fee(1_000 * PRECISION);

    stability_pool_utils::assert_pool_asset(
        &contracts.stability_pool,
        asset_with_fee_adjustment,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    // 3,000 initially deposited, 1000 used to pay off debt, 1,500 left in pool
    stability_pool_utils::assert_total_usdf_deposits(
        &contracts.stability_pool,
        3_000 * PRECISION - debt_paid_off,
    )
    .await;

    // Admin is 2/3 of the pool, depositor_2 is 1/6, depositor_3 is 1/6
    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        asset_with_fee_adjustment * 2 / 3,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    // Admin lost 2/3 of the usdf used to pay off debt
    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        2_000 * PRECISION - debt_paid_off * 2 / 3,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(depositor_2.address().into()),
        asset_with_fee_adjustment / 6,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(depositor_2.address().into()),
        500 * PRECISION - debt_paid_off / 6,
    )
    .await;
}

#[tokio::test]
async fn proper_no_reward_when_depositing_and_rewards_already_distributed() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;
    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let depositor_2 = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        2_000 * PRECISION,
    )
    .await
    .unwrap();

    let usdf_asset_id: AssetId = AssetId::from(*contracts.usdf.contract_id().hash());
    let tx_params = TxParameters::default().set_gas_price(1);

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
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

    return;

    admin
        .transfer(
            depositor_2.address().into(),
            500_000 * PRECISION,
            usdf_asset_id,
            tx_params,
        )
        .await
        .unwrap();

    let depositor_2_sp = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        depositor_2.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &depositor_2_sp,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        500_000 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(depositor_2.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(depositor_2.address().into()),
        500_000 * PRECISION,
    )
    .await;
}

#[tokio::test]
async fn proper_depositor_move_gain_to_trove() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;
    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        1_500 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
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

    let asset_with_fee = with_min_borrow_fee(1_050 * PRECISION);

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        asset_with_fee,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_abi::withdraw_gain_to_trove(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].active_pool,
        &contracts.asset_contracts[0].oracle,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
        6_000 * PRECISION + asset_with_fee,
    )
    .await;
}

#[tokio::test]
async fn proper_one_sp_depositor_position_multiple_assets() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, true).await;
    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;
    oracle_abi::set_price(&contracts.asset_contracts[1].oracle, 10 * PRECISION).await;

    let liquidated_wallet = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        6_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5_000 * PRECISION,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.asset_contracts[1].active_pool,
        6_000 * PRECISION,
        3_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.asset_contracts[1].active_pool,
        1_100 * PRECISION,
        1_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let init_stability_deposit = 3_000 * PRECISION;
    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        init_stability_deposit,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;
    oracle_abi::set_price(&contracts.asset_contracts[1].oracle, 1 * PRECISION).await;

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
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

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[1].trove_manager,
        &contracts.stability_pool,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].active_pool,
        &contracts.asset_contracts[1].default_pool,
        &contracts.asset_contracts[1].coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Since the entire debt is liquidated including the borrow fee,
    // the asset recieved includes the 0.5% fee
    let asset_with_fee_adjustment = with_min_borrow_fee(1_050 * PRECISION);
    let debt_with_fee_adjustment = with_min_borrow_fee(1_000 * PRECISION);

    stability_pool_utils::assert_pool_asset(
        &contracts.stability_pool,
        asset_with_fee_adjustment,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_total_usdf_deposits(
        &contracts.stability_pool,
        init_stability_deposit - 2 * debt_with_fee_adjustment,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        asset_with_fee_adjustment,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        asset_with_fee_adjustment,
        contracts.asset_contracts[1].asset.contract_id().into(),
    )
    .await;

    // 500 - 0.5% fee
    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        init_stability_deposit - 2 * debt_with_fee_adjustment,
    )
    .await;

    // Makes a 2nd deposit to the Stability Pool
    let second_deposit = 1_000 * PRECISION;

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        second_deposit,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        init_stability_deposit - 2 * debt_with_fee_adjustment + second_deposit,
    )
    .await;

    // Gain has been withdrawn and resset
    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
        contracts.asset_contracts[1].asset.contract_id().into(),
    )
    .await;

    let provider = admin.provider().unwrap();

    let fuel_asset_id: AssetId =
        AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(admin.address(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(fuel_balance, asset_with_fee_adjustment);

    let provider = admin.provider().unwrap();

    let st_fuel_asset_id: AssetId =
        AssetId::from(*contracts.asset_contracts[1].asset.contract_id().hash());

    let st_fuel_balance = provider
        .get_asset_balance(admin.address(), st_fuel_asset_id)
        .await
        .unwrap();

    assert_within_threshold(
        st_fuel_balance,
        asset_with_fee_adjustment,
        "st_fuel_balance not currect",
    );
}
