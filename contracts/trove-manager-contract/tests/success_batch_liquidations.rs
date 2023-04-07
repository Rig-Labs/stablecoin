use fuels::types::Identity;
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi,
        borrow_operations::borrow_operations_utils,
        coll_surplus_pool::coll_surplus_pool_abi,
        default_pool::default_pool_abi,
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, StabilityPool},
        trove_manager::{trove_manager_abi, trove_manager_utils, Status},
    },
    setup::common::setup_protocol,
    utils::{with_liquidation_penalty, with_min_borrow_fee},
};

#[tokio::test]
async fn proper_batch_liquidations_enough_usdf_in_sp() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, false).await;

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 10_000_000).await;

    let liquidated_wallet = wallets.pop().unwrap();
    let liquidated_wallet2 = wallets.pop().unwrap();
    let healthy_wallet1 = wallets.pop().unwrap();

    let usdf_deposit_to_be_liquidated = 1_000_000_000;
    let asset_deposit_to_be_liquidated = 1_100_000_000;

    borrow_operations_utils::mint_token_and_open_trove(
        liquidated_wallet.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        asset_deposit_to_be_liquidated,
        usdf_deposit_to_be_liquidated,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        liquidated_wallet2.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        asset_deposit_to_be_liquidated,
        usdf_deposit_to_be_liquidated,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet1.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        10_000_000_000,
        5_000_000_000,
    )
    .await;

    let stability_pool_healthy_wallet1 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_healthy_wallet1,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        5_000_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.asset_contracts[0].oracle, 1_000_000).await;
    // 2 wallets has collateral ratio of 110% and wallet 2 has 200% so we can liquidate it

    trove_manager_abi::batch_liquidate_troves(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].sorted_troves,
        &contracts.asset_contracts[0].active_pool,
        &contracts.asset_contracts[0].default_pool,
        &contracts.asset_contracts[0].coll_surplus_pool,
        &contracts.usdf,
        vec![
            Identity::Address(liquidated_wallet.address().into()),
            Identity::Address(liquidated_wallet2.address().into()),
        ],
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
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
    assert_eq!(deposits, 5_000_000_000 - 2 * liquidated_net_debt);

    let asset = stability_pool_abi::get_asset(
        &contracts.stability_pool,
        contracts.asset_contracts[0].asset.contract_id().into(),
    )
    .await
    .unwrap()
    .value;

    // 5% Penalty on 1_000_000_000 of debt
    let asset_with_min_borrow_fee = with_min_borrow_fee(1_050_000_000);
    assert_eq!(asset, 2 * asset_with_min_borrow_fee);

    let active_pool_asset = active_pool_abi::get_asset(&contracts.asset_contracts[0].active_pool)
        .await
        .value;

    let active_pool_debt =
        active_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].active_pool)
            .await
            .value;

    assert_eq!(active_pool_asset, 10_000_000_000);

    let active_pool_debt_with_min_borrow_fee = with_min_borrow_fee(5_000_000_000);
    assert_eq!(active_pool_debt, active_pool_debt_with_min_borrow_fee);

    let default_pool_asset =
        default_pool_abi::get_asset(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    let default_pool_debt =
        default_pool_abi::get_usdf_debt(&contracts.asset_contracts[0].default_pool)
            .await
            .value;

    assert_eq!(default_pool_asset, 0);
    assert_eq!(default_pool_debt, 0);

    let liq_coll_surplus = coll_surplus_pool_abi::get_collateral(
        &contracts.asset_contracts[0].coll_surplus_pool,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .value;

    // Prices are the same
    assert_eq!(
        liq_coll_surplus,
        asset_deposit_to_be_liquidated - with_liquidation_penalty(liquidated_net_debt),
        "Liquidated wallet collateral surplus was not 50_000"
    );
}
