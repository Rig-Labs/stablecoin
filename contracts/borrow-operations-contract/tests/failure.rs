use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::borrow_operations::borrow_operations_abi,
    interfaces::sorted_troves::sorted_troves_abi,
    interfaces::{active_pool::active_pool_abi, token::token_abi},
    interfaces::{trove_manager::trove_manager_abi, usdf_token::usdf_token_abi},
    setup::common::{deploy_token, deploy_usdf_token, setup_protocol},
    utils::{calculate_icr, with_min_borrow_fee},
};

#[tokio::test]
async fn fails_open_two_troves() {
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

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

    let res = borrow_operations_abi::open_trove(
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
    .is_err();

    assert!(res);

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
    assert_eq!(usdf_balance, 600 * PRECISION);

    let expected_debt = with_min_borrow_fee(600 * PRECISION);
    let expected_icr = calculate_icr(1_200 * PRECISION, expected_debt);

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

    assert_eq!(trove_col, 1_200 * PRECISION, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;
    assert_eq!(
        active_pool_col,
        1_200 * PRECISION,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn fails_open_trove_under_minimum_collateral_ratio() {
    // MCR = 1_200_000
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200 * PRECISION,
        1_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to open trove with MCR < 1.2"
    );
}

#[tokio::test]
async fn fails_open_trove_under_min_usdf_required() {
    // MCR = 1_200_000
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_200 * PRECISION,
        100 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to open trove with MCR < 1.2"
    );
}

#[tokio::test]
async fn fails_reduce_debt_under_min_usdf_required() {
    // MCR = 1_200_000
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

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

    let res = borrow_operations_abi::repay_usdf(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        300 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to reduce debt to less than 500 USDF"
    );
}

#[tokio::test]
async fn fails_decrease_collateral_under_mcr() {
    // MCR = 1_200_000
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

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

    let res = borrow_operations_abi::withdraw_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1_000 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to reduce collateral to less than 1.2 MCR"
    );
}

#[tokio::test]
async fn fails_incorrect_token_as_collateral_or_repayment() {
    // MCR = 1_200_000
    let (contracts, admin, _) = setup_protocol(100, 2, false).await;

    let mock_fake_token = deploy_token(&admin).await;

    token_abi::initialize(
        &mock_fake_token,
        0,
        &Identity::Address(admin.address().into()),
        "Fake Fuel".to_string(),
        "FFUEL".to_string(),
    )
    .await;

    token_abi::mint_to_id(
        &mock_fake_token,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &mock_fake_token,
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
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to open trove with incorrect token as collateral"
    );

    // Set up real trove and try to add collateral
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

    let res = borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &mock_fake_token,
        &contracts.usdf,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to add collateral with incorrect token as collateral"
    );

    let fake_usdf_token = deploy_usdf_token(&admin).await;

    usdf_token_abi::initialize(
        &fake_usdf_token,
        "Fake USDF".to_string(),
        "FUSDF".to_string(),
        fake_usdf_token.contract_id().into(),
        Identity::Address(admin.address().into()),
        Identity::Address(admin.address().into()),
    )
    .await;

    usdf_token_abi::mint(
        &fake_usdf_token,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::repay_usdf(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].asset,
        &fake_usdf_token,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.asset_contracts[0].active_pool,
        1 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to repay with incorrect token as repayment"
    );
}
