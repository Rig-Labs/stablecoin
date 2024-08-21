use fuels::prelude::*;

use test_utils::{
    interfaces::{
        oracle::{oracle_abi, Oracle},
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        redstone_oracle::{redstone_oracle_abi, redstone_price_feed},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_oracle},
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

    let pyth = deploy_mock_pyth_oracle(&wallet).await;
    let redstone = deploy_mock_redstone_oracle(&wallet).await;
    let instance = deploy_oracle(
        &wallet,
        pyth.contract_id().into(),
        redstone.contract_id().into(),
    )
    .await;

    // pyth_oracle_abi::update_price_feeds(
    //     &contracts.asset_contracts[0].mock_pyth_oracle,
    //     pyth_price_feed(1),
    // )
    // .await;

    // redstone_oracle_abi::write_prices(
    //     &contracts.asset_contracts[0].mock_redstone_oracle,
    //     redstone_price_feed(vec![1]),
    // )
    // .await;
    // redstone_oracle_abi::set_timestamp(
    //     &contracts.asset_contracts[0].mock_redstone_oracle,
    //     PYTH_TIMESTAMP,
    // )
    // .await;

    instance
}

#[ignore]
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
