use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        community_issuance::{community_issuance_abi, CommunityIssuance},
    },
    setup::common::{deploy_community_issuance},
};

async fn get_community_issuance () -> 
    CommunityIssuance<WalletUnlocked> {
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
    .await;
    let wallet = wallets.pop().unwrap();

    let instance = deploy_community_issuance(&wallet).await;

    instance
}

#[tokio::test]
async fn test_dec_pow() {
    let instance = get_community_issuance().await;

    let res = community_issuance_abi::test_dec_pow(
        &instance,
        999_998_681,
        2,
    ).await.value;

    println!("res {:?}", res);
}