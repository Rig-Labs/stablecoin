use fuels::prelude::*;

// Load abi from json
use test_utils::{
    interfaces::borrow_operations as borrow_operations_abi,
    interfaces::borrow_operations::BorrowOperations,
    interfaces::oracle as oracle_abi,
    interfaces::oracle::Oracle,
    interfaces::sorted_troves as sorted_troves_abi,
    interfaces::sorted_troves::SortedTroves,
    interfaces::token as token_abi,
    interfaces::token::Token,
    interfaces::trove_manager as trove_manager_abi,
    interfaces::trove_manager::TroveManagerContract,
    setup::common::{
        deploy_borrow_operations, deploy_oracle, deploy_sorted_troves, deploy_token,
        deploy_trove_manager_contract,
    },
};

async fn get_contract_instances() -> (
    BorrowOperations,
    TroveManagerContract,
    Oracle,
    SortedTroves,
    Token, /* Fuel */
    Token, /* USDF */
    WalletUnlocked,
) {
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

    let bo_instance = deploy_borrow_operations(&wallet).await;
    let oracle_instance = deploy_oracle(&wallet).await;
    let sorted_troves = deploy_sorted_troves(&wallet).await;
    let trove_manger = deploy_trove_manager_contract(&wallet).await;
    let fuel = deploy_token(&wallet).await;
    let usdf = deploy_token(&wallet).await;

    (
        bo_instance,
        trove_manger,
        oracle_instance,
        sorted_troves,
        fuel,
        usdf,
        wallets[0].clone(),
    )
}

#[tokio::test]
async fn can_set_and_retrieve_irc() {
    let (_instance, _admin, _, _, _, _, _) = get_contract_instances().await;

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
