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
};

#[tokio::test]
async fn test_permissions() {
    let (contracts, admin, mut wallets) = setup_protocol(2, false, false).await;

    let pauser = borrow_operations_abi::get_pauser(&contracts.borrow_operations)
        .await
        .unwrap();
    assert_eq!(
        pauser.value,
        Identity::Address(admin.address().into()),
        "Pauser should be the admin"
    );
    // Test setting pause status to true
    let _ = borrow_operations_abi::set_pause_status(&contracts.borrow_operations, true)
        .await
        .unwrap();

    let status = borrow_operations_abi::get_is_paused(&contracts.borrow_operations)
        .await
        .unwrap();
    assert!(status.value, "Failed to set pause status to true");

    // Test setting pause status to false
    let _ = borrow_operations_abi::set_pause_status(&contracts.borrow_operations, false)
        .await
        .unwrap();
    let status = borrow_operations_abi::get_is_paused(&contracts.borrow_operations)
        .await
        .unwrap();
    assert!(!status.value, "Failed to set pause status to false");

    let unauthorized_wallet = wallets.pop().unwrap();
    let unauthorized_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        unauthorized_wallet.clone(),
    );

    // Try to set pause status with unauthorized wallet
    let res = borrow_operations_abi::set_pause_status(&unauthorized_borrow_operations, true).await;

    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to set pause status"
    );
}

#[tokio::test]
async fn test_paused_operations() {
    let (contracts, admin, _) = setup_protocol(2, true, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[1].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let deposit_amount = 1_200 * PRECISION;
    let borrow_amount = 600 * PRECISION;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    // Open a trove while unpaused
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
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Set pause status to true
    borrow_operations_abi::set_pause_status(&contracts.borrow_operations, true)
        .await
        .unwrap();

    // Try to open another trove while paused
    let res = borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[1].oracle,
        &contracts.asset_contracts[1].mock_pyth_oracle,
        &contracts.asset_contracts[1].mock_redstone_oracle,
        &contracts.asset_contracts[1].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[1].trove_manager,
        &contracts.active_pool,
        deposit_amount,
        borrow_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_err(),
        "Should not be able to open trove while paused"
    );

    // Try to withdraw USDF (increase debt) while paused
    let withdraw_amount = 100 * PRECISION;
    let res = borrow_operations_abi::withdraw_usdf(
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
        withdraw_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_err(),
        "Should not be able to withdraw USDF while paused"
    );

    // Try to repay USDF (reduce debt) while paused
    let repay_amount = 100 * PRECISION;
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
        repay_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(res.is_ok(), "Should be able to repay USDF while paused");

    // Try to add collateral while paused
    let res = borrow_operations_abi::add_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(res.is_ok(), "Should be able to add collateral while paused");

    // Try to withdraw collateral while paused
    let res = borrow_operations_abi::withdraw_coll(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_ok(),
        "Should be able to withdraw collateral while paused"
    );

    // Set pause status to false
    borrow_operations_abi::set_pause_status(&contracts.borrow_operations, false)
        .await
        .unwrap();

    // Try to withdraw USDF while unpaused
    let res = borrow_operations_abi::withdraw_usdf(
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
        withdraw_amount,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await;

    assert!(
        res.is_ok(),
        "Should be able to withdraw USDF while unpaused"
    );
}
