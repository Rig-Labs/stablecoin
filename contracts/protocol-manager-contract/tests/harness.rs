use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::active_pool::{active_pool_abi, ActivePool},
    interfaces::token::{token_abi, Token},
    setup::common::{deploy_active_pool, deploy_default_pool, deploy_token},
};

async fn get_contract_instance() -> (ActivePool, Token, WalletUnlocked) {
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

    let instance = deploy_active_pool(&wallet).await;
    let default_pool = deploy_default_pool(&wallet).await;

    let asset = deploy_token(&wallet).await;

    token_abi::initialize(
        &asset,
        1_000_000_000,
        &Identity::Address(wallet.address().into()),
        "Fuel".to_string(),
        "FUEL".to_string(),
    )
    .await;

    active_pool_abi::initialize(
        &instance,
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
        asset.contract_id().into(),
        default_pool.contract_id().into(),
    )
    .await;

    (instance, asset, wallet)
}

#[tokio::test]
async fn proper_intialize() {}
