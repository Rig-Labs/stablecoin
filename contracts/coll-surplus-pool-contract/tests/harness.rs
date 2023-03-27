use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::coll_surplus_pool::{coll_surplus_pool_abi, CollSurplusPool},
    interfaces::token::{token_abi, Token},
    setup::common::{deploy_coll_surplus_pool, deploy_token},
};

async fn get_contract_instance() -> (CollSurplusPool, Token, WalletUnlocked) {
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

    let coll_pool = deploy_coll_surplus_pool(&wallet).await;

    let asset = deploy_token(&wallet).await;

    token_abi::initialize(
        &asset,
        1_000_000_000,
        &Identity::Address(wallet.address().into()),
        "Fuel".to_string(),
        "FUEL".to_string(),
    )
    .await;

    coll_surplus_pool_abi::initialize(
        &coll_pool,
        Identity::Address(wallet.address().into()),
        coll_pool.contract_id().into(),
        asset.contract_id().into(),
        asset.contract_id().into(),
    )
    .await;

    (coll_pool, asset, wallet)
}

#[tokio::test]
async fn proper_intialize() {
    let (coll_surplus_pool, mock_fuel, admin) = get_contract_instance().await;

    let coll = coll_surplus_pool_abi::get_asset(&coll_surplus_pool, mock_fuel.contract_id().into())
        .await
        .value;
    assert_eq!(coll, 0);

    let balance = coll_surplus_pool_abi::get_collateral(
        &coll_surplus_pool,
        Identity::Address(admin.address().into()),
        mock_fuel.contract_id().into(),
    )
    .await;

    assert_eq!(balance.value, 0);
}

#[tokio::test]
async fn proper_adjust_debt() {
    // TODO
}

#[tokio::test]
async fn proper_adjust_asset_col() {
    // TODO
}
