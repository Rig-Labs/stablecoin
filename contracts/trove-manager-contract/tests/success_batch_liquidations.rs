use fuels::prelude::*;
use fuels::types::Identity;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::borrow_operations_utils,
        coll_surplus_pool::coll_surplus_pool_abi,
        default_pool::default_pool_abi,
        oracle::oracle_abi,
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed, pyth_price_feed_with_time, PYTH_TIMESTAMP,
        },
        stability_pool::{stability_pool_abi, StabilityPool},
        trove_manager::{trove_manager_abi, trove_manager_utils, Status},
    },
    setup::common::setup_protocol,
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn proper_batch_liquidations_enough_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

    let liquidated_wallet = wallets.pop().unwrap();
    let liquidated_wallet2 = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();

    let usdf_deposit_to_be_liquidated = 1_000 * PRECISION;
    let asset_deposit_to_be_liquidated = 1_100 * PRECISION;

    borrow_operations_utils::mint_token_and_open_trove(
        liquidated_wallet.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        asset_deposit_to_be_liquidated,
        usdf_deposit_to_be_liquidated,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        liquidated_wallet2.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        asset_deposit_to_be_liquidated,
        usdf_deposit_to_be_liquidated,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet1.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        10_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    let stability_pool_healthy_wallet1 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_healthy_wallet1,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
    )
    .await
    .unwrap();

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed_with_time(1, PYTH_TIMESTAMP + 1),
    )
    .await;
    // 2 wallets has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::batch_liquidate_troves(
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
        vec![
            Identity::Address(liquidated_wallet.address().into()),
            Identity::Address(liquidated_wallet2.address().into()),
        ],
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet2.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet2.address().into()),
        0,
    )
    .await;

    trove_manager_utils::assert_trove_status(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(liquidated_wallet2.address().into()),
        Status::ClosedByLiquidation,
    )
    .await;

    let deposits = stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    let liquidated_net_debt = with_min_borrow_fee(usdf_deposit_to_be_liquidated);
    assert_eq!(deposits, 5_000 * PRECISION - 2 * liquidated_net_debt);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset_id,
    )
    .await
    .unwrap()
    .value;

    // 5% Penalty on 1_000* PRECISION of debt
    let asset_with_min_borrow_fee = 1_100 * PRECISION;
    let coll_gas_fee = asset_with_min_borrow_fee / 200;

    assert_eq!(asset, 2 * (asset_with_min_borrow_fee - coll_gas_fee));

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;

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

    assert_eq!(active_pool_asset, 10_000 * PRECISION);

    let active_pool_debt_with_min_borrow_fee = with_min_borrow_fee(5_000 * PRECISION);
    assert_eq!(active_pool_debt, active_pool_debt_with_min_borrow_fee);

    let default_pool_asset = default_pool_abi::get_asset(
        &contracts.default_pool,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;

    let default_pool_debt = default_pool_abi::get_usdf_debt(
        &contracts.default_pool,
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
        contracts.asset_contracts[0]
            .asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;

    // all collateral is liquidated with no surplus
    assert_eq!(
        liq_coll_surplus, 0,
        "Liquidated wallet collateral surplus was not 0"
    );
}
