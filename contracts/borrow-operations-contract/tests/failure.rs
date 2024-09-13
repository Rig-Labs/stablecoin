use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::borrow_operations_abi,
        oracle::oracle_abi,
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        sorted_troves::sorted_troves_abi,
        token::token_abi,
        trove_manager::trove_manager_abi,
        usdf_token::usdf_token_abi,
    },
    setup::common::{deploy_token, deploy_usdf_token, setup_protocol},
    utils::{calculate_icr, with_min_borrow_fee},
};

#[tokio::test]
async fn fails_open_two_troves_of_same_coll_type() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

    let col_amount = 1_200 * PRECISION;
    let debt_amount = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        col_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        col_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .is_err();

    assert!(res);

    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            contracts
                .usdf
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
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
    assert_eq!(usdf_balance, debt_amount);

    let expected_debt = with_min_borrow_fee(debt_amount);
    let expected_icr = calculate_icr(col_amount, expected_debt);

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

    assert_eq!(trove_col, col_amount, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col, col_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn fails_open_trove_under_minimum_collateral_ratio() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    // 134.9% Collateral Ratio < 135% Minimum Collateral Ratio
    let coll_amount = 1349 * PRECISION;
    let debt_amount = 1000 * PRECISION;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        coll_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_err(),
        "Borrow operation: Should not be able to open trove with MCR < 135%"
    );
}

#[tokio::test]
async fn fails_open_trove_under_min_usdf_required() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let coll_amount = 1_200 * PRECISION;
    let debt_amount = 400 * PRECISION;
    // 100 USDF < 500 USDF
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        coll_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_err(),
        "Borrow operation: Should not be able to open trove with debt < 500 USDF"
    );
}

#[tokio::test]
async fn fails_reduce_debt_under_min_usdf_required() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let coll_amount = 1_200 * PRECISION;
    let debt_amount = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        coll_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();
    println!("Opened trove");

    // 600 USDF - 300 USDF < 500 USDF

    let res = borrow_operations_abi::repay_usdf(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        300 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .is_err();
    println!("Repaying");

    assert!(
        res,
        "Borrow operation: Should not be able to reduce debt to less than 500 USDF"
    );
}

#[tokio::test]
async fn fails_decrease_collateral_under_mcr() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let coll_amount = 1_200 * PRECISION;
    let debt_amount = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        coll_amount,
        debt_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::withdraw_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        600 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to reduce collateral to less than 130% MCR"
    );
}

#[tokio::test]
async fn fails_incorrect_token_as_collateral_or_repayment() {
    let (contracts, admin, _) = setup_protocol(100, 2, false, false).await;

    let mock_fake_token = deploy_token(&admin).await;

    token_abi::initialize(
        &mock_fake_token,
        0,
        &Identity::Address(admin.address().into()),
        "Fake Coll".to_string(),
        "FCOL".to_string(),
    )
    .await
    .unwrap();

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

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &mock_fake_token,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let res = borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &mock_fake_token,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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
        fake_usdf_token.contract_id().into(),
        Identity::Address(admin.address().into()),
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap();

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
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &fake_usdf_token,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .is_err();

    assert!(
        res,
        "Borrow operation: Should not be able to repay with incorrect token as repayment"
    );
}
