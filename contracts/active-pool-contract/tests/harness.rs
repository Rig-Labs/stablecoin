use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::ContractInstance,
    interfaces::{
        active_pool::{active_pool_abi, ActivePool},
        token::{token_abi, Token},
    },
    setup::common::{deploy_active_pool, deploy_default_pool, deploy_token},
};

async fn get_contract_instance() -> (
    ContractInstance<ActivePool<WalletUnlocked>>,
    Token<WalletUnlocked>,
    WalletUnlocked,
) {
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
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();

    let instance = deploy_active_pool(&wallet).await;
    let default_pool = deploy_default_pool(&wallet).await;

    let asset = deploy_token(&wallet).await;

    token_abi::initialize(
        &asset,
        1_000_000_000,
        &Identity::Address(wallet.address().into()),
        "Mock".to_string(),
        "MOCK".to_string(),
    )
    .await
    .unwrap();

    active_pool_abi::initialize(
        &instance,
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        default_pool.contract.contract_id().into(),
        Identity::Address(wallet.address().into()),
    )
    .await
    .unwrap();

    active_pool_abi::add_asset(
        &instance,
        asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
        Identity::Address(wallet.address().into()),
    )
    .await;

    (instance, asset, wallet)
}

#[tokio::test]
async fn proper_intialize() {
    let (active_pool, mock_fuel, _admin) = get_contract_instance().await;

    let debt = active_pool_abi::get_usdm_debt(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(debt, 0);

    let asset_amount = active_pool_abi::get_asset(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(asset_amount, 0);
}

#[tokio::test]
async fn proper_adjust_debt() {
    let (active_pool, mock_fuel, _admin) = get_contract_instance().await;

    active_pool_abi::increase_usdm_debt(
        &active_pool,
        1000,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let debt = active_pool_abi::get_usdm_debt(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(debt, 1000);

    active_pool_abi::decrease_usdm_debt(
        &active_pool,
        500,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let debt = active_pool_abi::get_usdm_debt(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(debt, 500);
}

#[tokio::test]
async fn proper_adjust_asset_col() {
    let (active_pool, mock_fuel, admin) = get_contract_instance().await;

    token_abi::mint_to_id(
        &mock_fuel,
        1_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    active_pool_abi::recieve(&active_pool, &mock_fuel, 1_000_000).await;

    let asset_amount = active_pool_abi::get_asset(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(asset_amount, 1_000_000);

    let provdier = admin.provider().unwrap();

    let asset_id = mock_fuel
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    let balance_before = provdier
        .get_asset_balance(admin.address().into(), asset_id)
        .await
        .unwrap();

    active_pool_abi::send_asset(
        &active_pool,
        Identity::Address(admin.address().into()),
        500_000,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let asset_amount = active_pool_abi::get_asset(
        &active_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(asset_amount, 500_000);

    let balance_after = provdier
        .get_asset_balance(admin.address().into(), asset_id)
        .await
        .unwrap();

    assert_eq!(balance_before + 500_000, balance_after);
}
