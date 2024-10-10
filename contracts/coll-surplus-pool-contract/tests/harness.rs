use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        coll_surplus_pool::{coll_surplus_pool_abi, CollSurplusPool},
        oracle::oracle_abi,
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed, pyth_price_feed_with_time, PYTH_PRECISION,
            PYTH_TIMESTAMP,
        },
        token::token_abi,
        trove_manager::trove_manager_abi,
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn test_collateral_surplus_workflow_after_liquidation() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;
    // Set up liquidated wallet's trove
    let collateral = 600 * PRECISION;
    let debt = 500 * PRECISION;
    let liquidated_wallet = wallets.pop().unwrap();
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        collateral,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    let borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        collateral,
        debt,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // At least one healthy trove needed for liquidation
    let healthy_wallet = wallets.pop().unwrap();
    let healthy_wallet_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        healthy_wallet.clone(),
    );

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        collateral,
        Identity::Address(healthy_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &healthy_wallet_borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        collateral,
        500 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Simulate price drop
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed_with_time(1, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
    )
    .await;

    // Perform liquidation
    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    // Check for collateral surplus
    let surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .unwrap()
    .value;

    assert!(surplus > 0, "No collateral surplus after liquidation");

    // Claim the surplus
    borrow_operations_abi::claim_coll(
        &borrow_operations,
        &contracts.active_pool,
        &contracts.coll_surplus_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await;

    let provdier = liquidated_wallet.provider().unwrap();

    let asset_id = contracts.asset_contracts[0]
        .asset
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    let balance_after = provdier
        .get_asset_balance(liquidated_wallet.address().into(), asset_id)
        .await
        .unwrap();

    assert_eq!(balance_after, surplus, "Surplus not zero after claiming");

    let final_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(final_surplus, 0, "Surplus not zero after claiming");
}

#[tokio::test]
async fn test_coll_surplus_pool_access_control() {
    let (contracts, admin, mut wallets) = setup_protocol(3, false, false).await;
    let coll_surplus_pool = CollSurplusPool::new(
        contracts.coll_surplus_pool.contract_id().clone(),
        admin.clone(),
    );
    let unauthorized_wallet = wallets.pop().unwrap();

    // Test initialize access control
    let result = coll_surplus_pool_abi::initialize(
        &CollSurplusPool::new(
            coll_surplus_pool.contract_id().clone(),
            unauthorized_wallet.clone(),
        ),
        ContractId::from([0u8; 32]),
        Identity::Address(unauthorized_wallet.address().into()),
    )
    .await;
    assert!(
        result.is_err(),
        "Unauthorized wallet should not be able to initialize twice"
    );

    // Test add_asset access control
    let result = coll_surplus_pool_abi::add_asset(
        &CollSurplusPool::new(
            coll_surplus_pool.contract_id().clone(),
            unauthorized_wallet.clone(),
        ),
        AssetId::default(),
        Identity::Address(unauthorized_wallet.address().into()),
    )
    .await;
    assert!(
        result.is_err(),
        "Unauthorized wallet should not be able to add asset"
    );

    // Test claim_coll access control
    let result = coll_surplus_pool_abi::claim_coll(
        &CollSurplusPool::new(
            coll_surplus_pool.contract_id().clone(),
            unauthorized_wallet.clone(),
        ),
        Identity::Address(unauthorized_wallet.address().into()),
        &contracts.active_pool,
        AssetId::default(),
    )
    .await;
    assert!(
        result.is_err(),
        "Unauthorized wallet should not be able to claim collateral"
    );

    // Test account_surplus access control
    let result = coll_surplus_pool_abi::account_surplus(
        &CollSurplusPool::new(
            coll_surplus_pool.contract_id().clone(),
            unauthorized_wallet.clone(),
        ),
        Identity::Address(unauthorized_wallet.address().into()),
        100,
        AssetId::default(),
    )
    .await;
    assert!(
        result.is_err(),
        "Unauthorized wallet should not be able to account surplus"
    );

    // Test get_asset and get_collateral accessibility
    let result = coll_surplus_pool_abi::get_asset(&coll_surplus_pool, AssetId::default()).await;
    assert!(result.is_ok(), "get_asset should be accessible");

    let result = coll_surplus_pool_abi::get_collateral(
        &coll_surplus_pool,
        Identity::Address(Address::from([0u8; 32])),
        AssetId::default(),
    )
    .await;
    assert!(result.is_ok(), "get_collateral should be accessible");
}
