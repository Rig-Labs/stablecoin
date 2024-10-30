use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        token::token_abi,
    },
    setup::common::setup_protocol,
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn test_trove_events() {
    let (contracts, admin, wallets) = setup_protocol(4, false, false).await;

    // Setup initial conditions
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let deposit_amount = 1200 * PRECISION;
    let borrow_amount = 600 * PRECISION;
    let additional_collateral = 300 * PRECISION;

    // Set oracle price
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    // Test OpenTroveEvent
    let response = borrow_operations_abi::open_trove(
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
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let logs = response.decode_logs();
    let open_trove_event = logs
        .results
        .iter()
        .find(|log| log.as_ref().unwrap().contains("OpenTroveEvent"))
        .expect("OpenTroveEvent not found")
        .as_ref()
        .unwrap();

    assert!(
        open_trove_event.contains(&admin.address().hash().to_string()),
        "OpenTroveEvent should contain user address"
    );
    assert!(
        open_trove_event.contains(&deposit_amount.to_string()),
        "OpenTroveEvent should contain collateral amount"
    );
    assert!(
        open_trove_event.contains(&with_min_borrow_fee(borrow_amount).to_string()),
        "OpenTroveEvent should contain debt amount"
    );
    assert!(
        open_trove_event.contains(&contracts.asset_contracts[0].asset_id.to_string()),
        "OpenTroveEvent should contain asset id"
    );

    // Test AdjustTroveEvent
    let response = borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        additional_collateral,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let logs = response.decode_logs();
    let adjust_event = logs
        .results
        .iter()
        .find(|log| log.as_ref().unwrap().contains("AdjustTroveEvent"))
        .expect("AdjustTroveEvent not found")
        .as_ref()
        .unwrap();
    println!("adjust_event: {:?}", adjust_event);
    assert!(
        adjust_event.contains(&admin.address().hash().to_string()),
        "AdjustTroveEvent should contain user address"
    );
    assert!(
        adjust_event.contains(&additional_collateral.to_string()),
        "AdjustTroveEvent should contain collateral change amount"
    );
    assert!(
        adjust_event.contains("is_collateral_increase: true"),
        "AdjustTroveEvent should indicate collateral increase"
    );
    assert!(
        adjust_event.contains("is_debt_increase: false"),
        "AdjustTroveEvent should indicate debt is not increased"
    );
    // assetid
    assert!(
        adjust_event.contains(&contracts.asset_contracts[0].asset_id.to_string()),
        "AdjustTroveEvent should contain asset id"
    );
    // total debt
    assert!(
        adjust_event.contains(&with_min_borrow_fee(borrow_amount).to_string()),
        "AdjustTroveEvent should contain total debt"
    );
    // total coll
    assert!(
        adjust_event.contains(&(deposit_amount + additional_collateral).to_string()),
        "AdjustTroveEvent should contain total collateral"
    );

    // create one more trove to allow for closing

    let second_wallet = wallets[1].clone();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5000 * PRECISION,
        Identity::Address(second_wallet.address().into()),
    )
    .await;

    let borrow_operations_second_wallet = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        second_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_second_wallet,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
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

    second_wallet
        .transfer(
            &admin.address(),
            borrow_amount,
            contracts.usdf_asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Test CloseTroveEvent
    let response = borrow_operations_abi::close_trove(
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
        with_min_borrow_fee(borrow_amount),
    )
    .await
    .unwrap();

    let logs = response.decode_logs();
    let close_event = logs
        .results
        .iter()
        .find(|log| log.as_ref().unwrap().contains("CloseTroveEvent"))
        .expect("CloseTroveEvent not found")
        .as_ref()
        .unwrap();

    assert!(
        close_event.contains(&admin.address().hash().to_string()),
        "CloseTroveEvent should contain user address"
    );
    assert!(
        close_event.contains(&contracts.asset_contracts[0].asset_id.to_string()),
        "CloseTroveEvent should contain asset id"
    );
    assert!(
        close_event.contains(&(deposit_amount + additional_collateral).to_string()),
        "CloseTroveEvent should contain collateral amount"
    );
    assert!(
        close_event.contains(&with_min_borrow_fee(borrow_amount).to_string()),
        "CloseTroveEvent should contain debt amount"
    );
}
