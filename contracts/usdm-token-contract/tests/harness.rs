use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::{ContractInstance, PRECISION},
    interfaces::{
        active_pool::{active_pool_abi, ActivePool},
        default_pool::{default_pool_abi, DefaultPool},
        token::{token_abi, Token},
        usdm_token::{usdm_token_abi, USDMToken},
    },
    setup::common::{deploy_active_pool, deploy_default_pool, deploy_token, setup_protocol},
};

async fn get_contract_instance() -> (
    ContractInstance<DefaultPool<WalletUnlocked>>,
    Token<WalletUnlocked>,
    WalletUnlocked,
    ContractInstance<ActivePool<WalletUnlocked>>,
) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),                 /* Single wallet */
            Some(1),                 /* Single coin (UTXO) */
            Some(1_000 * PRECISION), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();

    let default_pool = deploy_default_pool(&wallet).await;
    let active_pool = deploy_active_pool(&wallet).await;

    let asset = deploy_token(&wallet).await;

    token_abi::initialize(
        &asset,
        1_000 * PRECISION,
        &Identity::Address(wallet.address().into()),
        "Mock".to_string(),
        "MOCK".to_string(),
    )
    .await
    .unwrap();

    default_pool_abi::initialize(
        &default_pool,
        Identity::Address(wallet.address().into()),
        active_pool.contract.contract_id().into(),
    )
    .await
    .unwrap();

    active_pool_abi::initialize(
        &active_pool,
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        default_pool.contract.contract_id().into(),
        Identity::Address(wallet.address().into()),
    )
    .await
    .unwrap();

    active_pool_abi::add_asset(
        &active_pool,
        asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
        Identity::Address(wallet.address().into()),
    )
    .await;

    default_pool_abi::add_asset(
        &default_pool,
        asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
        Identity::Address(wallet.address().into()),
    )
    .await;

    (default_pool, asset, wallet, active_pool)
}

#[tokio::test]
async fn proper_intialize() {
    let (default_pool, mock_fuel, _admin, _) = get_contract_instance().await;

    let debt = default_pool_abi::get_usdm_debt(
        &default_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(debt, 0);

    let asset_amount = default_pool_abi::get_asset(
        &default_pool,
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
    let (default_pool, mock_fuel, _admin, _) = get_contract_instance().await;

    default_pool_abi::increase_usdm_debt(
        &default_pool,
        1000,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let debt = default_pool_abi::get_usdm_debt(
        &default_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(debt, 1000);

    default_pool_abi::decrease_usdm_debt(
        &default_pool,
        500,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let debt = default_pool_abi::get_usdm_debt(
        &default_pool,
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
    let (default_pool, mock_fuel, admin, active_pool) = get_contract_instance().await;

    token_abi::mint_to_id(
        &mock_fuel,
        1 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    active_pool_abi::recieve(&active_pool, &mock_fuel, 1 * PRECISION).await;

    active_pool_abi::send_asset_to_default_pool(
        &active_pool,
        &default_pool,
        &mock_fuel,
        1 * PRECISION,
    )
    .await
    .unwrap();

    let asset_amount = default_pool_abi::get_asset(
        &default_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(asset_amount, 1 * PRECISION);

    default_pool_abi::send_asset_to_active_pool(
        &default_pool,
        &active_pool,
        PRECISION / 2,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await;

    let asset_amount = default_pool_abi::get_asset(
        &default_pool,
        mock_fuel
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into(),
    )
    .await
    .value;
    assert_eq!(asset_amount, PRECISION / 2);
}

#[tokio::test]
async fn fails_unauthorized_usdm_operations() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;

    let attacker = wallets.pop().unwrap();

    // Create a new instance of the USDM token contract with the attacker's wallet
    let usdm_token_attacker = ContractInstance::new(
        USDMToken::new(contracts.usdm.contract.contract_id(), attacker.clone()),
        contracts.usdm.implementation_id,
    );

    // Try to add a trove manager using the attacker's wallet
    let result =
        usdm_token_abi::add_trove_manager(&usdm_token_attacker, ContractId::from([1u8; 32])).await;

    // Assert that the operation fails
    assert!(
        result.is_err(),
        "Unauthorized user should not be able to add a trove manager"
    );

    // Optionally, you can check for a specific error message
    if let Err(error) = result {
        assert!(
            error.to_string().contains("USDMToken: NotAuthorized"),
            "Unexpected error message: {}",
            error
        );
    }

    // Create a new instance of the USDM token contract with the attacker's wallet
    let usdm_token_attacker = ContractInstance::new(
        USDMToken::new(contracts.usdm.contract.contract_id(), attacker.clone()),
        contracts.usdm.implementation_id,
    );

    // Test 1: Unauthorized add_trove_manager
    let result =
        usdm_token_abi::add_trove_manager(&usdm_token_attacker, ContractId::from([1u8; 32])).await;

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to add a trove manager"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("USDMToken: NotAuthorized"),
            "Unexpected error message: {}",
            error
        );
    }

    // Test 2: Unauthorized mint
    let result = usdm_token_abi::mint(
        &usdm_token_attacker,
        1_000 * PRECISION,
        Identity::Address(attacker.address().into()),
    )
    .await;

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to mint USDM tokens"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("USDMToken: NotAuthorized"),
            "Unexpected error message: {}",
            error
        );
    }
}
