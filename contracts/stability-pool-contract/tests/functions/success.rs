use crate::utils::setup::setup;
use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, stability_pool_utils},
        token::token_abi,
        trove_manager::trove_manager_abi,
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_initialization() {
    let (stability_pool, _, _, _, _) = setup(Some(4)).await;

    stability_pool_utils::assert_pool_asset(&stability_pool, 0).await;

    stability_pool_utils::assert_total_usdf_deposits(&stability_pool, 0).await;
}

#[tokio::test]
async fn proper_stability_deposit() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4).await;

    token_abi::mint_to_id(
        &contracts.fuel,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        600_000_000,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_pool_asset(&contracts.stability_pool, 0).await;

    stability_pool_utils::assert_total_usdf_deposits(&contracts.stability_pool, 600_000_000).await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        600_000_000,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
    )
    .await;
}

#[tokio::test]
async fn proper_stability_widthdrawl() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4).await;

    token_abi::mint_to_id(
        &contracts.fuel,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        600_000_000,
    )
    .await
    .unwrap();

    stability_pool_abi::withdraw_from_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        &contracts.fuel,
        300_000_000,
    )
    .await
    .unwrap();

    stability_pool_utils::assert_pool_asset(&contracts.stability_pool, 0).await;

    stability_pool_utils::assert_total_usdf_deposits(&contracts.stability_pool, 300_000_000).await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        300_000_000,
    )
    .await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        0,
    )
    .await;
}

#[tokio::test]
async fn proper_liquidation_distribution_one_sp_depositor() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4).await;
    oracle_abi::set_price(&contracts.oracle, 10_000_000).await;

    let liquidated_wallet = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.fuel,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &contracts.fuel,
        5_000_000_000,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        3_000_000_000,
        1_500_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let liq_borrow_operations = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        liquidated_wallet.clone(),
    );

    borrow_operations_abi::open_trove(
        &liq_borrow_operations,
        &contracts.oracle,
        &contracts.fuel,
        &contracts.usdf,
        &contracts.sorted_troves,
        &contracts.trove_manager,
        &contracts.active_pool,
        1_100_000_000,
        1_000_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.usdf,
        1_500_000_000,
    )
    .await
    .unwrap();

    oracle_abi::set_price(&contracts.oracle, 1_000_000).await;

    trove_manager_abi::liquidate(
        &contracts.trove_manager,
        &contracts.stability_pool,
        &contracts.oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(liquidated_wallet.address().into()),
    )
    .await
    .unwrap();

    stability_pool_utils::assert_pool_asset(&contracts.stability_pool, 1_050_000_000).await;

    stability_pool_utils::assert_total_usdf_deposits(&contracts.stability_pool, 500_000_000).await;

    stability_pool_utils::assert_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        1_050_000_000,
    )
    .await;

    stability_pool_utils::assert_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
        500_000_000,
    )
    .await;
}
