use crate::utils::setup::setup;
use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::{
        borrow_operations::borrow_operations_abi, stability_pool::stability_pool_abi,
        token::token_abi,
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_initialization() {
    let (stability_pool, _, _, _, _) = setup(Some(4)).await;

    let asset_amount = stability_pool_abi::get_asset(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset_amount, 0);

    let total_usdf_deposits = stability_pool_abi::get_total_usdf_deposits(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(total_usdf_deposits, 0);
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

    let asset_amount = stability_pool_abi::get_asset(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset_amount, 0);

    let total_usdf_deposits =
        stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
            .await
            .unwrap()
            .value;

    assert_eq!(total_usdf_deposits, 600_000_000);

    let compounded_usdf = stability_pool_abi::get_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(compounded_usdf, 600_000_000);

    let gain = stability_pool_abi::get_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(gain, 0);
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

    let asset_amount = stability_pool_abi::get_asset(&contracts.stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset_amount, 0);

    let total_usdf_deposits =
        stability_pool_abi::get_total_usdf_deposits(&contracts.stability_pool)
            .await
            .unwrap()
            .value;

    assert_eq!(total_usdf_deposits, 300_000_000);

    let compounded_usdf = stability_pool_abi::get_compounded_usdf_deposit(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(compounded_usdf, 300_000_000);

    let gain = stability_pool_abi::get_depositor_asset_gain(
        &contracts.stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(gain, 0);
}
