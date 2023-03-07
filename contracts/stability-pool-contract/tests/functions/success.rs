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
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel,
        usdf,
        active_pool,
        admin,
        _wallets,
        stability_pool,
    ) = setup_protocol(10, 4).await;

    token_abi::mint_to_id(
        &fuel,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations,
        &oracle,
        &fuel,
        &usdf,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        0,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(&stability_pool, &usdf, 600_000_000)
        .await
        .unwrap();

    let asset_amount = stability_pool_abi::get_asset(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset_amount, 0);

    let total_usdf_deposits = stability_pool_abi::get_total_usdf_deposits(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(total_usdf_deposits, 600_000_000);

    let compounded_usdf = stability_pool_abi::get_compounded_usdf_deposit(
        &stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(compounded_usdf, 600_000_000);

    let gain = stability_pool_abi::get_depositor_asset_gain(
        &stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(gain, 0);
}

#[tokio::test]
async fn proper_stability_widthdrawl() {
    let (
        borrow_operations,
        trove_manager,
        oracle,
        sorted_troves,
        fuel,
        usdf,
        active_pool,
        admin,
        _wallets,
        stability_pool,
    ) = setup_protocol(10, 4).await;

    token_abi::mint_to_id(
        &fuel,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations,
        &oracle,
        &fuel,
        &usdf,
        &sorted_troves,
        &trove_manager,
        &active_pool,
        0,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(&stability_pool, &usdf, 600_000_000)
        .await
        .unwrap();

    stability_pool_abi::withdraw_from_stability_pool(&stability_pool, &usdf, &fuel, 300_000_000)
        .await
        .unwrap();

    let asset_amount = stability_pool_abi::get_asset(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(asset_amount, 0);

    let total_usdf_deposits = stability_pool_abi::get_total_usdf_deposits(&stability_pool)
        .await
        .unwrap()
        .value;

    assert_eq!(total_usdf_deposits, 300_000_000);

    let compounded_usdf = stability_pool_abi::get_compounded_usdf_deposit(
        &stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(compounded_usdf, 300_000_000);

    let gain = stability_pool_abi::get_depositor_asset_gain(
        &stability_pool,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap()
    .value;

    assert_eq!(gain, 0);
}
