use fuels::{prelude::AssetId, types::Identity};
use test_utils::data_structures::PRECISION;
use test_utils::interfaces::protocol_manager::ProtocolManager;
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        coll_surplus_pool::coll_surplus_pool_abi,
        oracle::oracle_abi,
        protocol_manager::protocol_manager_abi,
        token::token_abi,
        trove_manager::{trove_manager_utils, Status, trove_manager_abi},
    },
    setup::common::setup_protocol,
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn proper_redemption_from_partially_closed() {

    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, true).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    let balance = 10_000 * PRECISION;

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

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
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

    let borrow_operations_healthy_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet2.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        9_000 * PRECISION,
        5_000 * PRECISION,
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
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        8_000 * PRECISION,
        5_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    let redemption_amount: u64 = 3_000 * PRECISION;

    let protocol_manager_health1 = ProtocolManager::new(
        contracts.protocol_manager.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    let pre_redemption_active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    protocol_manager_abi::redeem_collateral(
        &protocol_manager_health1,
        redemption_amount,
        10,
        0,
        0,
        None,
        None,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts,
    )
    .await;

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    assert_eq!(active_pool_asset, 24_000 * PRECISION);

    assert_eq!(
        active_pool_debt,
        pre_redemption_active_pool_debt - redemption_amount
    );

    let provider = healthy_wallet1.provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(healthy_wallet1.address(), fuel_asset_id)
        .await
        .unwrap();

    // TODO Replace with staking contract when implemented
    let oracle_balance = provider
        .get_contract_asset_balance(
            contracts.asset_contracts[0].oracle.contract_id(),
            fuel_asset_id,
        )
        .await
        .unwrap();


    // here we need to calculate the fee and subtract it
    let redemption_asset_fee = trove_manager_abi::get_redemption_fee(
        &contracts.asset_contracts[0].trove_manager,
        redemption_amount
    )
    .await.value;

    assert_eq!(fuel_balance, redemption_amount - oracle_balance - redemption_asset_fee);

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        5_000 * PRECISION,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        with_min_borrow_fee(5_000 * PRECISION) - 3_000 * PRECISION,
    )
    .await;
}

#[tokio::test]
async fn proper_redemption_with_a_trove_closed_fully() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, true).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10 * PRECISION).await;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    let balance: u64 = 12_000 * PRECISION;

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

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(healthy_wallet3.address().into()),
    )
    .await;

    let borrow_operations_healthy_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    let coll1 = 12_000 * PRECISION;
    let debt1 = 6_000 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        coll1,
        debt1,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let borrow_operations_healthy_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet2.clone(),
    );

    let coll2: u64 = 9_000 * PRECISION;
    let debt2: u64 = 5_000 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        coll2,
        debt2,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let borrow_operations_healthy_wallet3 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet3.clone(),
    );

    let coll3: u64 = 8_000 * PRECISION;
    let debt3: u64 = 5_000 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet3,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        coll3,
        debt3,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    // Troves
    // H1: 12/6 = 2 -> H2: 9/5 = 1.8 -> H3: 8/5 = 1.6

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1 * PRECISION).await;

    let redemption_amount: u64 = 6_000 * PRECISION;

    let protocol_manager_health1 = ProtocolManager::new(
        contracts.protocol_manager.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    protocol_manager_abi::redeem_collateral(
        &protocol_manager_health1,
        redemption_amount,
        3,
        0,
        0,
        None,
        None,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.asset_contracts,
    )
    .await;

    println!("Collateral redeemed");

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    let collateral_taken_from_trove3 = with_min_borrow_fee(5_000 * PRECISION);
    let remaining_collateral_to_redeem = redemption_amount - collateral_taken_from_trove3;

    assert_eq!(
        active_pool_asset,
        coll1 + coll2 - remaining_collateral_to_redeem
    );

    let total_debt = with_min_borrow_fee(debt1 + debt2 + debt3);
    assert_eq!(active_pool_debt, total_debt - redemption_amount);

    let provider = healthy_wallet1.provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(healthy_wallet1.address(), fuel_asset_id)
        .await
        .unwrap();

    let staking_balance = provider
        .get_contract_asset_balance(
            &contracts.fpt_staking.contract_id(),
            fuel_asset_id,
        )
        .await
        .unwrap();

    assert_eq!(fuel_balance, 6_000 * PRECISION - staking_balance);

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        Status::ClosedByRedemption,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        Status::Active,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        9_000 * PRECISION - remaining_collateral_to_redeem,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet2.address().into()),
        with_min_borrow_fee(debt2) - remaining_collateral_to_redeem,
    )
    .await;

    let coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.asset_contracts[0].coll_surplus_pool,
        Identity::Address(healthy_wallet3.address().into()),
    )
    .await
    .value;

    assert_eq!(coll_surplus, coll3 - with_min_borrow_fee(debt3));
}
