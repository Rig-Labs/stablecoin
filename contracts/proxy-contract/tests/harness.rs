use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::proxy::{proxy_abi, Proxy, State},
    setup::common::deploy_proxy,
};

const DEFAULT_TARGET_CONTRACT_ID: [u8; 32] = [1u8; 32];

async fn get_contract_instance() -> (Proxy<Wallet>, Wallet, Wallet) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Two wallets */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet2 = wallets.pop().unwrap();
    let wallet = wallets.pop().unwrap();

    let instance = deploy_proxy(
        ContractId::from(DEFAULT_TARGET_CONTRACT_ID),
        wallet.clone(),
        None,
    )
    .await;

    (instance, wallet, wallet2)
}

#[tokio::test]
async fn test_initial_state() {
    let (proxy, _wallet, _wallet2) = get_contract_instance().await;

    let target = proxy_abi::get_proxy_target(&proxy).await.unwrap().value;
    assert_eq!(target, Some(ContractId::from(DEFAULT_TARGET_CONTRACT_ID)));

    let owner = proxy_abi::get_proxy_owner(&proxy).await.unwrap().value;
    assert_eq!(
        owner,
        State::Initialized(Identity::Address(_wallet.address().into()))
    );
}

#[tokio::test]
async fn test_unauthorized_set_target() {
    let (proxy, _wallet, wallet2) = get_contract_instance().await;

    // Try to set target with unauthorized wallet
    let result =
        proxy_abi::set_proxy_target(&proxy.with_account(wallet2), ContractId::from([2u8; 32]))
            .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_unauthorized_set_owner() {
    let (proxy, _wallet, wallet2) = get_contract_instance().await;

    // Try to set owner with unauthorized wallet
    let result = proxy_abi::set_proxy_owner(
        &proxy.with_account(wallet2.clone()),
        State::Initialized(Identity::Address(wallet2.address().into())),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_authorized_operations() {
    let (proxy, wallet, _wallet2) = get_contract_instance().await;

    // Set initial owner
    proxy_abi::set_proxy_owner(
        &proxy,
        State::Initialized(Identity::Address(wallet.address().into())),
    )
    .await
    .unwrap();

    let owner = proxy_abi::get_proxy_owner(&proxy).await.unwrap().value;
    assert_eq!(
        owner,
        State::Initialized(Identity::Address(wallet.address().into()))
    );

    // Set target
    let target = ContractId::from([3u8; 32]);
    proxy_abi::set_proxy_target(&proxy, target).await.unwrap();

    let stored_target = proxy_abi::get_proxy_target(&proxy).await.unwrap().value;
    assert_eq!(stored_target, Some(target));
}
