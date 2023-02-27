use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::trove_manager::TroveManagerContract, setup::common::deploy_trove_manager_contract,
};
// Load abi from json

async fn get_contract_instance() -> (TroveManagerContract, WalletUnlocked) {
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

    let instance = deploy_trove_manager_contract(&wallet).await;

    (instance, wallets[0].clone())
}

#[tokio::test]
async fn can_set_and_retrieve_irc() {
    // let (instance, admin) = get_contract_instance().await;
    // let irc: u64 = 100;
    // // Increment the counter
    // let _result = instance
    //     .methods()
    //     .set_nominal_icr(Identity::Address(admin.address().into()), irc)
    //     .call()
    //     .await
    //     .unwrap();

    // // Get the current value of the counter
    // let result = instance
    //     .methods()
    //     .get_nominal_icr(Identity::Address(admin.address().into()))
    //     .call()
    //     .await
    //     .unwrap();

    // // Check that the current value of the counter is 1.
    // // Recall that the initial value of the counter was 0.
    // assert_eq!(result.value, irc);

    // Now you have an instance of your contract you can use to test each function
}
