use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        community_issuance::{community_issuance_abi, CommunityIssuance},
    },
    setup::common::{deploy_community_issuance},
};

async fn get_community_issuance () -> (
    CommunityIssuance<WalletUnlocked>,
    WalletUnlocked,
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
    .await;
    let wallet = wallets.pop().unwrap();

    let instance = deploy_community_issuance(&wallet).await;

    (instance, wallet)
}

#[tokio::test]
async fn test_emissions() {
    let (instance, wallet) = get_community_issuance().await;

    // here let's basically just set one year in seconds difference and see what the issuance is. because my unit test didn't work

    // community_issuance_abi::initialize(
    //     &instance, 
    //     instance.contract_id().into(), 
    //     instance.contract_id().into(), 
    //     wallet.address().into(), 
    //     true, 
    //     0).await;

    // let fraction = community_issuance_abi::get_cumulative_issuance_fraction(
    //     &instance,
    //     60 * 60 * 24 * 30,
    //     0
    // ).await.value;

    // println!("done {:?}", fraction);

    let current_time = 60 * 60 * 24 * 30 * 12;
    let deployment_time = 0;
    let time_transition_started = current_time - (current_time / 24);
    let total_transition_time_seconds = current_time / 6;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = true;

    println!("time since transition started {:?}", current_time - time_transition_started);
    println!("total transition time {}", total_transition_time_seconds);
    println!("change in fpt supply cap {:?}", (current_time - time_transition_started) / total_transition_time_seconds);

    let res = community_issuance_abi::external_test_issue_fpt(
        &instance,
        current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards
    ).await.value;

    println!("res {:?}", res);
}