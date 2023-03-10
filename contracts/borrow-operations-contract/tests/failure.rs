use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::borrow_operations::borrow_operations_abi,
    interfaces::sorted_troves::sorted_troves_abi,
    interfaces::trove_manager::trove_manager_abi,
    interfaces::{active_pool::active_pool_abi, token::token_abi},
    setup::common::{deploy_token, setup_protocol},
};

#[tokio::test]
async fn fails_open_two_troves() {
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(res);

    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            AssetId::from(*usdf_token.contract_id().hash()),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(&sorted_troves).await.value;
    let last = sorted_troves_abi::get_last(&sorted_troves).await.value;
    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;
    let icr = trove_manager_abi::get_nominal_icr(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 600_000_000);

    assert_eq!(icr, 2_000_000_000, "ICR is wrong");

    let trove_col = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, 600_000_000, "Trove Debt is wrong");

    let active_pool_debt = active_pool_abi::get_usdf_debt(&active_pool).await.value;
    assert_eq!(active_pool_debt, 600_000_000, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(&active_pool).await.value;
    assert_eq!(
        active_pool_col, 1_200_000_000,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn fails_open_trove_under_minimum_collateral_ratio() {
    // MCR = 1_200_000
    // TODO update this if MCR changes
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        1_000_000_000,
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
    // TODO update this if MCR changes
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        100_000_000,
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
    // TODO update this if MCR changes
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::repay_usdf(
        &borrow_operations,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        300_000_000,
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
    // TODO update this if MCR changes
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::withdraw_coll(
        &borrow_operations,
        &oracle,
        &fuel_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_000_000_000,
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
    // TODO update this if MCR changes
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        active_pool,
        admin,
        _,
        _,
    ) = setup_protocol(100, 2).await;

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
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &fuel_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &borrow_operations,
        &oracle,
        &mock_fake_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
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
        &borrow_operations,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::add_coll(
        &borrow_operations,
        &oracle,
        &mock_fake_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to add collateral with incorrect token as collateral"
    );

    let fake_usdf_token = deploy_token(&admin).await;

    token_abi::initialize(
        &fake_usdf_token,
        0,
        &Identity::Address(admin.address().into()),
        "Fake USDF".to_string(),
        "FUSDF".to_string(),
    )
    .await;

    token_abi::mint_to_id(
        &fake_usdf_token,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let res = borrow_operations_abi::repay_usdf(
        &borrow_operations,
        &oracle,
        &fuel_token,
        &fake_usdf_token,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        1_000_000,
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
