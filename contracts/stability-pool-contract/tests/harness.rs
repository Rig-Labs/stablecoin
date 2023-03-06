use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::{
        borrow_operations::borrow_operations_abi,
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
    },
    setup::common::{deploy_stability_pool, setup_protocol},
};

async fn get_contract_instance() -> (WalletUnlocked, StabilityPool) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let stability_pool = deploy_stability_pool(&wallet).await;

    (wallet, stability_pool)
}

#[tokio::test]
async fn proper_initialization() {
    let (_, stability_pool) = get_contract_instance().await;

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
}
