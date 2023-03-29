use fuels::{prelude::*, types::Identity};

// Load abi from json
use test_utils::{
    interfaces::borrow_operations::borrow_operations_abi,
    interfaces::borrow_operations::BorrowOperations,
    interfaces::sorted_troves::sorted_troves_abi,
    interfaces::trove_manager::trove_manager_abi,
    interfaces::{active_pool::active_pool_abi, token::token_abi},
    setup::common::setup_protocol,
    utils::{calculate_icr, with_min_borrow_fee},
};

#[tokio::test]
async fn proper_creating_trove() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    println!("{:?}", res);

    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            AssetId::from(*contracts.usdf.contract_id().hash()),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 600_000_000);

    let expected_net_debt: u64 = with_min_borrow_fee(usdf_balance);
    let expected_icr = calculate_icr(1_200_000_000, expected_net_debt);

    assert_eq!(icr, expected_icr, "ICR is wrong");

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(
        active_pool_debt, expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col, 1_200_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_increase_collateral() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000_000_000,
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
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(600_000_000);

    assert_eq!(trove_col, 2_400_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    let expected_nicr = calculate_icr(2_400_000_000, expected_debt);

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col, 2_400_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_decrease_collateral() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let balance = 5_000_000_000;
    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let _ = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    borrow_operations_abi::withdraw_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        30_000_0000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(600_000_000);

    assert_eq!(trove_col, 900_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_nicr = calculate_icr(900_000_000, expected_debt);

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(admin_balance, 4_100_000_000, "Balance is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col, 900_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_increase_debt() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let balance = 5_000_000_000;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());
    let usdf_asset_id = AssetId::from(*contracts.usdf.contract_id().hash());

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::withdraw_usdf(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        200_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(800_000_000);

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_nicr = calculate_icr(1_200_000_000, expected_debt);

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(admin_balance, 3_800_000_000, "Balance is wrong");

    let usdf_balance = provider
        .get_asset_balance(admin.address().into(), usdf_asset_id)
        .await
        .unwrap();

    assert_eq!(usdf_balance, 800_000_000, "USDF Balance is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    assert_eq!(
        active_pool_col, 1_200_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_decrease_debt() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let balance = 5_000_000_000;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());
    let usdf_asset_id = AssetId::from(*contracts.usdf.contract_id().hash());

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        800_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::repay_usdf(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        200_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(800_000_000) - 200_000_000;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    let expected_nicr = calculate_icr(1_200_000_000, expected_debt);

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(admin_balance, 3_800_000_000, "Balance is wrong");

    let usdf_balance = provider
        .get_asset_balance(admin.address().into(), usdf_asset_id)
        .await
        .unwrap();

    assert_eq!(usdf_balance, 600_000_000, "USDF Balance is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    assert_eq!(
        active_pool_col, 1_200_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_open_multiple_troves() {
    let (contracts, _admin, mut wallets) = setup_protocol(100, 4, false).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 5_000_000_000;
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
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        3_000_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        2_000_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;

    assert_eq!(size, 2);
    assert_eq!(first, Identity::Address(wallet1.address().into()));
    assert_eq!(last, Identity::Address(wallet2.address().into()));

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    let expected_debt = with_min_borrow_fee(2_000_000_000);
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    assert_eq!(
        active_pool_col, 5_000_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_close_trove() {
    let (contracts, admin, mut wallets) = setup_protocol(100, 4, false).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 5_000_000_000;
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
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        3_000_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let usdf_asset_id: AssetId = AssetId::from(*contracts.usdf.contract_id().hash());
    let amount = 1_000_000_000 / 200;
    let tx_parms = TxParameters::default();

    wallet1
        .transfer(wallet2.address(), amount, usdf_asset_id, tx_parms)
        .await
        .unwrap();

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        2_000_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let expected_debt = with_min_borrow_fee(1_000_000_000);

    borrow_operations_abi::close_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        expected_debt,
    )
    .await;

    let provider = admin.get_provider().unwrap();
    let fuel_asset_id: AssetId =
        AssetId::from(*contracts.asset_contracts[0].asset.contract_id().hash());

    let wallet2_balance = provider
        .get_asset_balance(&wallet2.address(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(wallet2_balance, 5_000_000_000, "Wallet 2 balance is wrong");

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(wallet1.address().into()));
    assert_eq!(last, Identity::Address(wallet1.address().into()));

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    assert_eq!(
        active_pool_col, 3_000_000_000,
        "Active Pool Collateral is wrong"
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        2_000_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();
    // Can open a new trove after closing one
}

#[tokio::test]
async fn proper_creating_trove_with_2nd_asset() {
    let (contracts, admin, mut wallets) = setup_protocol(100, 2, true).await;

    let wallet2 = wallets.pop().unwrap();

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            AssetId::from(*contracts.usdf.contract_id().hash()),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let last = sorted_troves_abi::get_last(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[0].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 600_000_000);

    let expected_net_debt: u64 = with_min_borrow_fee(usdf_balance);
    let expected_icr = calculate_icr(1_200_000_000, expected_net_debt);

    assert_eq!(icr, expected_icr, "ICR is wrong");

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(
        active_pool_debt, expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col, 1_200_000_000,
        "Active Pool Collateral is wrong"
    );

    // ------- 2nd asset -------- //
    // Minting with stFUEL //

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5_000_000_000,
        Identity::Address(wallet2.address().into()),
    )
    .await;

    let borrow_operations_wallet2 =
        BorrowOperations::new(contracts.borrow_operations.contract_id().clone(), wallet2);

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.asset_contracts[1].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let provider = admin.get_provider().unwrap();

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.asset_contracts[1].active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            AssetId::from(*contracts.usdf.contract_id().hash()),
        )
        .await
        .unwrap();

    let size = sorted_troves_abi::get_size(&contracts.asset_contracts[1].sorted_troves)
        .await
        .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[1].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 2);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 2 * 600_000_000);

    let expected_net_debt: u64 = with_min_borrow_fee(600_000_000);
    let expected_icr = calculate_icr(1_200_000_000, expected_net_debt);

    assert_eq!(icr, expected_icr, "ICR is wrong");

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[1].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[1].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[1].active_pool)
            .await
            .value;
    assert_eq!(
        active_pool_debt,
        2 * expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[1].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col,
        2 * 1_200_000_000,
        "Active Pool Collateral is wrong"
    );

    borrow_operations_abi::close_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.asset_contracts[1].sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.asset_contracts[1].active_pool,
        with_min_borrow_fee(600_000_000),
    )
    .await;
}
