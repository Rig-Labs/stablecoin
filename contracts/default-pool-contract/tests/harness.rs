use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::default_pool::{default_pool_abi, DefaultPool},
    interfaces::token::{token_abi, Token},
    setup::common::{deploy_default_pool, deploy_token},
};

async fn get_contract_instance() -> (DefaultPool, Token, WalletUnlocked) {
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
        Identity::Address(wallet.address().into()),
        asset.contract_id().into(),
    )
    .await;

    (instance, asset, wallet)
}

#[tokio::test]
async fn proper_intialize() {
    let (default_pool, _mock_fuel, _admin) = get_contract_instance().await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 0);

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 0);
}

#[tokio::test]
async fn proper_adjust_debt() {
    let (default_pool, _mock_fuel, _admin) = get_contract_instance().await;

    default_pool_abi::increase_usdf_debt(&default_pool, 1000).await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 1000);

    default_pool_abi::decrease_usdf_debt(&default_pool, 500).await;

    let debt = default_pool_abi::get_usdf_debt(&default_pool).await.value;
    assert_eq!(debt, 500);
}

#[tokio::test]
async fn proper_adjust_asset_col() {
    let (default_pool, mock_fuel, admin) = get_contract_instance().await;

    token_abi::mint_to_id(
        &mock_fuel,
        1_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    default_pool_abi::recieve(&default_pool, &mock_fuel, 1_000_000).await;

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 1_000_000);

    let provdier = admin.get_provider().unwrap();

    let asset_id = AssetId::from(*mock_fuel.contract_id().hash());
    let balance_before = provdier
        .get_asset_balance(admin.address().into(), asset_id)
        .await
        .unwrap();

    default_pool_abi::send_asset_to_active_pool(&default_pool, 500_000).await;

    let asset_amount = default_pool_abi::get_asset(&default_pool).await.value;
    assert_eq!(asset_amount, 500_000);

    let balance_after = provdier
        .get_asset_balance(admin.address().into(), asset_id)
        .await
        .unwrap();

    assert_eq!(balance_before + 500_000, balance_after);
}
