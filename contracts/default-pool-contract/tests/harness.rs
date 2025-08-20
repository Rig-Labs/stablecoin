use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::ContractInstance,
    interfaces::usdm_token::{usdm_token_abi, USDMToken},
    setup::common::deploy_usdm_token,
};

async fn get_contract_instance() -> (ContractInstance<USDMToken<Wallet>>, Wallet, Vec<Wallet>) {
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

    let asset = deploy_usdm_token(&wallet).await;

    usdm_token_abi::initialize(
        &asset,
        asset.contract.contract_id().into(),
        Identity::Address(wallet.address().into()),
        Identity::Address(wallet.address().into()),
    )
    .await
    .unwrap();

    (asset, wallet, wallets)
}

#[tokio::test]
async fn proper_intialize() {
    let (usdm, _admin, _) = get_contract_instance().await;

    let total_supply = usdm_token_abi::total_supply(&usdm).await.value.unwrap();

    assert_eq!(total_supply, 0);
}

#[tokio::test]
async fn proper_mint() {
    let (usdm, _, mut wallets) = get_contract_instance().await;

    let wallet = wallets.pop().unwrap();

    usdm_token_abi::mint(&usdm, 100, Identity::Address(wallet.address().into()))
        .await
        .unwrap();

    let total_supply = usdm_token_abi::total_supply(&usdm).await.value.unwrap();

    assert_eq!(total_supply, 100);
}

#[tokio::test]
async fn proper_burn() {
    let (usdm, admin, _wallets) = get_contract_instance().await;

    usdm_token_abi::mint(&usdm, 100, Identity::Address(admin.address().into()))
        .await
        .unwrap();

    let total_supply = usdm_token_abi::total_supply(&usdm).await.value.unwrap();

    assert_eq!(total_supply, 100);

    usdm_token_abi::burn(&usdm, 50).await.unwrap();

    let total_supply = usdm_token_abi::total_supply(&usdm).await.value.unwrap();

    assert_eq!(total_supply, 50);
}

#[tokio::test]
async fn fails_to_mint_unauthorized() {
    let (usdm, _, mut wallets) = get_contract_instance().await;

    let wallet = wallets.pop().unwrap();

    let unauthorized_usdm = ContractInstance::new(
        USDMToken::new(usdm.contract.contract_id(), wallet.clone()),
        usdm.implementation_id.clone(),
    );

    usdm_token_abi::mint(
        &unauthorized_usdm,
        100,
        Identity::Address(wallet.address().into()),
    )
    .await
    .expect_err("Should fail to mint");
}

#[tokio::test]
async fn fails_to_burn_unauthorized() {
    let (usdm, _, mut wallets) = get_contract_instance().await;

    let wallet = wallets.pop().unwrap();

    usdm_token_abi::mint(&usdm, 100, Identity::Address(wallet.address().into()))
        .await
        .unwrap();

    let unauthorized_usdm = ContractInstance::new(
        USDMToken::new(usdm.contract.contract_id(), wallet.clone()),
        usdm.implementation_id.clone(),
    );

    usdm_token_abi::burn(&unauthorized_usdm, 100)
        .await
        .expect_err("Should fail to burn");
}
