use fuels::prelude::*;

use test_utils::{
    interfaces::oracle::{oracle_abi, Oracle},
    setup::common::deploy_oracle,
};

async fn get_contract_instance() -> Oracle<WalletUnlocked> {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();

    let instance = deploy_oracle(&wallet).await;

    instance
}

#[tokio::test]
async fn can_set_proper_price() {
    let instance = get_contract_instance().await;
    let new_price: u64 = 100;
    // Increment the counter
    let _result = oracle_abi::set_price(&instance, new_price).await;

    // Get the current value of the counter
    let result = oracle_abi::get_price(&instance).await;

    // Check that the current value of the counter is 1.
    // Recall that the initial value of the counter was 0.
    assert_eq!(result.value, new_price);

    // Now you have an instance of your contract you can use to test each function
}
