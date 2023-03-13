use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::default_pool::{default_pool_abi, DefaultPool},
    interfaces::{
        active_pool::{active_pool_abi, ActivePool},
        token::{token_abi, Token},
    },
    setup::common::{deploy_active_pool, deploy_default_pool, deploy_token},
};

async fn get_contract_instance() -> (DefaultPool, Token, WalletUnlocked, ActivePool) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let instance = deploy_default_pool(&wallet).await;
    let active_pool = deploy_active_pool(&wallet).await;

    let asset = deploy_token(&wallet).await;

    token_abi::initialize(
        &asset,
        1_000_000_000,
        &Identity::Address(wallet.address().into()),
        "Fuel".to_string(),
        "FUEL".to_string(),
    )
    .await;

    default_pool_abi::initialize(
        &instance,
        Identity::Address(wallet.address().into()),
        active_pool.contract_id().into(),
        asset.contract_id().into(),
    )
    .await;

    active_pool_abi::initialize(
        &active_pool,
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        asset.contract_id().into(),
        instance.contract_id().into(),
    )
    .await;

    (instance, asset, wallet, active_pool)
}

#[tokio::test]
async fn proper_intialize() {
    let (default_pool, _mock_fuel, _admin, _) = get_contract_instance().await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 0);

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 0);
}

#[tokio::test]
async fn proper_adjust_debt() {
    let (default_pool, _mock_fuel, _admin, _) = get_contract_instance().await;

    default_pool_abi::increase_usdf_debt(&default_pool, 1000).await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 1000);

    default_pool_abi::decrease_usdf_debt(&default_pool, 500).await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 500);
}

#[tokio::test]
async fn proper_adjust_asset_col() {
    let (default_pool, mock_fuel, admin, active_pool) = get_contract_instance().await;

    token_abi::mint_to_id(
        &mock_fuel,
        1_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    active_pool_abi::recieve(&active_pool, &mock_fuel, 1_000_000).await;

    active_pool_abi::send_asset_to_default_pool(&active_pool, &default_pool, &mock_fuel, 1_000_000)
        .await
        .unwrap();

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 1_000_000);

    default_pool_abi::send_asset_to_active_pool(&default_pool, &active_pool, 500_000).await;

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 500_000);
}
