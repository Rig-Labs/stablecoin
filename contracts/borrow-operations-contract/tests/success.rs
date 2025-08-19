use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::{ContractInstance, PRECISION},
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        sorted_troves::sorted_troves_abi,
        token::token_abi,
        trove_manager::trove_manager_abi,
    },
    setup::common::setup_protocol,
    utils::{calculate_icr, with_min_borrow_fee},
};

#[tokio::test]
async fn proper_creating_trove() {
    let (contracts, admin, _) = setup_protocol(2, false, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();
    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 600 * PRECISION;

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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    println!("borrow_operations_abi::open_trove done");

    let usdm_balance = provider
        .get_asset_balance(
            admin.address().into(),
            contracts
                .usdm
                .contract
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
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
    assert_eq!(usdm_balance, borrow_amount);

    let expected_net_debt: u64 = with_min_borrow_fee(usdm_balance);
    let expected_icr = calculate_icr(deposit_amount, expected_net_debt);

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

    assert_eq!(trove_col, deposit_amount, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_debt, expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col, deposit_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_increase_collateral() {
    let (contracts, admin, _) = setup_protocol(2, false, false).await;

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 600 * PRECISION;

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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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

    let expected_debt = with_min_borrow_fee(borrow_amount);

    assert_eq!(trove_col, 2 * deposit_amount, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
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

    let expected_nicr = calculate_icr(2 * deposit_amount, expected_debt);

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col,
        2 * deposit_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_decrease_collateral() {
    let (contracts, admin, _) = setup_protocol(2, false, false).await;

    let balance = 5000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    let _ = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    let withdraw_amount = 300 * PRECISION;

    borrow_operations_abi::withdraw_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        withdraw_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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

    let expected_debt = with_min_borrow_fee(borrow_amount);

    assert_eq!(
        trove_col,
        deposit_amount - withdraw_amount,
        "Trove Collateral is wrong"
    );
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_nicr = calculate_icr(deposit_amount - withdraw_amount, expected_debt);

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(
            admin.address().into(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    assert_eq!(admin_balance, 4100 * PRECISION, "Balance is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col,
        deposit_amount - withdraw_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_increase_debt() {
    let (contracts, admin, _) = setup_protocol(2, false, false).await;

    let balance = 5000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

    let usdm_asset_id: AssetId = contracts.usdm_asset_id;

    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 600 * PRECISION;

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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    borrow_operations_abi::withdraw_usdm(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        200 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
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

    let expected_debt = with_min_borrow_fee(800 * PRECISION);

    assert_eq!(trove_col, deposit_amount, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let icr = trove_manager_abi::get_nominal_icr(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_nicr = calculate_icr(deposit_amount, expected_debt);

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(
            admin.address().into(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    assert_eq!(admin_balance, 3800 * PRECISION, "Balance is wrong");

    let usdm_balance = provider
        .get_asset_balance(admin.address().into(), usdm_asset_id)
        .await
        .unwrap();

    assert_eq!(usdm_balance, 800 * PRECISION, "USDM Balance is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;

    assert_eq!(
        active_pool_col, deposit_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_decrease_debt() {
    let (contracts, admin, _) = setup_protocol(2, false, false).await;

    let balance = 5000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

    let usdm_asset_id = contracts.usdm_asset_id;

    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 800 * PRECISION;

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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let repay_amount = 200 * PRECISION;
    borrow_operations_abi::repay_usdm(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        &contracts.default_pool,
        repay_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let trove_col = trove_manager_abi::get_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let _asset: ContractId = contracts.asset_contracts[0].asset.contract_id().into();

    let trove_debt = trove_manager_abi::get_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(borrow_amount) - repay_amount;

    assert_eq!(trove_col, deposit_amount, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_debt, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
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

    let expected_nicr = calculate_icr(deposit_amount, expected_debt);

    assert_eq!(icr, expected_nicr, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(
            admin.address().into(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    assert_eq!(admin_balance, 3800 * PRECISION, "Balance is wrong");

    let usdm_balance = provider
        .get_asset_balance(admin.address().into(), usdm_asset_id)
        .await
        .unwrap();

    assert_eq!(usdm_balance, 600 * PRECISION, "USDM Balance is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(
        active_pool_col, deposit_amount,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_open_multiple_troves() {
    let (contracts, _admin, mut wallets) = setup_protocol(4, false, false).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 5000 * PRECISION;
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

    let borrow_operations_wallet1 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            wallet1.clone(),
        ),
        contracts.borrow_operations.implementation_id.clone(),
    );

    let borrow_operations_wallet2 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            wallet2.clone(),
        ),
        contracts.borrow_operations.implementation_id.clone(),
    );

    let deposit_amount1 = 3000 * PRECISION;
    let borrow_amount1 = 1000 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount1,
        borrow_amount1,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let deposit_amount2 = 2000 * PRECISION;
    let borrow_amount2 = 1000 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount2,
        borrow_amount2,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(size, 2);
    assert_eq!(first, Identity::Address(wallet1.address().into()));
    assert_eq!(last, Identity::Address(wallet2.address().into()));

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let expected_debt = with_min_borrow_fee(borrow_amount1 + borrow_amount2);
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    assert_eq!(
        active_pool_col,
        deposit_amount1 + deposit_amount2,
        "Active Pool Collateral is wrong"
    );
}

#[tokio::test]
async fn proper_close_trove() {
    let (contracts, admin, mut wallets) = setup_protocol(4, false, false).await;

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let balance = 5000 * PRECISION;
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

    let borrow_operations_wallet1 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            wallet1.clone(),
        ),
        contracts.borrow_operations.implementation_id.clone(),
    );

    let borrow_operations_wallet2 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            wallet2.clone(),
        ),
        contracts.borrow_operations.implementation_id.clone(),
    );

    let deposit_amount1 = 3000 * PRECISION;
    let borrow_amount1 = 1000 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount1,
        borrow_amount1,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Transfering to cover the fee
    let usdm_asset_id: AssetId = contracts.usdm_asset_id;
    let amount = borrow_amount1 / 200;
    let tx_parms = TxPolicies::default()
        .with_tip(1)
        .with_script_gas_limit(2000000);

    wallet1
        .transfer(wallet2.address(), amount, usdm_asset_id, tx_parms)
        .await
        .unwrap();

    let deposit_amount2 = 2000 * PRECISION;
    let borrow_amount2 = 1000 * PRECISION;
    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount2,
        borrow_amount2,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let expected_debt = with_min_borrow_fee(borrow_amount2);

    let _res = borrow_operations_abi::close_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        expected_debt,
    )
    .await;
    // print_response(&res);

    let provider = admin.provider().unwrap();

    let wallet2_balance = provider
        .get_asset_balance(&wallet2.address(), contracts.asset_contracts[0].asset_id)
        .await
        .unwrap();

    assert_eq!(
        wallet2_balance,
        5000 * PRECISION,
        "Wallet 2 balance is wrong"
    );

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;

    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;
    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(wallet1.address().into()));
    assert_eq!(last, Identity::Address(wallet1.address().into()));

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;
    assert_eq!(active_pool_debt, expected_debt, "Active Pool Debt is wrong");

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .value;

    assert_eq!(
        active_pool_col,
        3000 * PRECISION,
        "Active Pool Collateral is wrong"
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        2000 * PRECISION,
        1000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();
    // Can open a new trove after closing one
}

#[tokio::test]
async fn proper_creating_trove_with_2nd_asset() {
    let (contracts, admin, mut wallets) = setup_protocol(2, true, false).await;

    let wallet2 = wallets.pop().unwrap();

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let provider = admin.provider().unwrap();

    let deposit_amount1 = 1200 * PRECISION;
    let borrow_amount1 = 600 * PRECISION;

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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        deposit_amount1,
        borrow_amount1,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let usdm_balance = provider
        .get_asset_balance(admin.address().into(), contracts.usdm_asset_id)
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    let last = sorted_troves_abi::get_last(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[0].asset_id.into(),
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
    assert_eq!(usdm_balance, borrow_amount1);

    let expected_net_debt: u64 = with_min_borrow_fee(usdm_balance);
    let expected_icr = calculate_icr(deposit_amount1, expected_net_debt);

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

    assert_eq!(trove_col, deposit_amount1, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_debt, expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col, deposit_amount1,
        "Active Pool Collateral is wrong"
    );

    // ------- 2nd asset -------- //

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let _ = token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5000 * PRECISION,
        Identity::Address(wallet2.address().into()),
    )
    .await;

    let borrow_operations_wallet2 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            wallet2.clone(),
        ),
        contracts.borrow_operations.implementation_id.clone(),
    );

    let deposit_amount2 = 1200 * PRECISION;
    let borrow_amount2 = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[1].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[1].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].mock_pyth_oracle,
        &contracts.asset_contracts[1].mock_redstone_oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.active_pool,
        deposit_amount2,
        borrow_amount2,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let provider = admin.provider().unwrap();

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].mock_pyth_oracle,
        &contracts.asset_contracts[1].mock_redstone_oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.active_pool,
        deposit_amount1,
        borrow_amount1,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let usdm_balance = provider
        .get_asset_balance(admin.address().into(), contracts.usdm_asset_id)
        .await
        .unwrap();

    let size = sorted_troves_abi::get_size(
        &contracts.sorted_troves,
        contracts.asset_contracts[1].asset_id.into(),
    )
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
    assert_eq!(usdm_balance, 2 * borrow_amount1);

    let expected_net_debt: u64 = with_min_borrow_fee(borrow_amount1);
    let expected_icr = calculate_icr(deposit_amount1, expected_net_debt);

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

    assert_eq!(trove_col, deposit_amount1, "Trove Collateral is wrong");
    assert_eq!(trove_debt, expected_net_debt, "Trove Debt is wrong");

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[1].asset_id,
    )
    .await
    .value;
    assert_eq!(
        active_pool_debt,
        2 * expected_net_debt,
        "Active Pool Debt is wrong"
    );

    let active_pool_col = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[1].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(
        active_pool_col,
        2 * deposit_amount1,
        "Active Pool Collateral is wrong"
    );

    // let usdm_asset_id =
    //     borrow_operations_abi::get_usdm_asset_id(&contracts.borrow_operations).await;

    // println!("USDM Asset ID: {:?}", usdm_asset_id);

    // let hex_string = usdm_asset_id.to_hex();

    // println!("{:?}", hex_string);

    println!("Expected: {:?}", contracts.usdm_asset_id);

    let _res = borrow_operations_abi::close_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].mock_pyth_oracle,
        &contracts.asset_contracts[1].mock_redstone_oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.active_pool,
        with_min_borrow_fee(borrow_amount1),
    )
    .await;
    // print_response(&res);
}
