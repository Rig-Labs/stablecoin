use fuels::{prelude::*, tx::ContractId};

// Load abi from json
abigen!(
    MyContract,
    "contracts/mock-oracle-contract/out/debug/mock-oracle-contract-abi.json"
);

// get path
fn get_path(sub_path: String) -> String {
    let mut path = std::env::current_dir().unwrap();
    path.push(sub_path);
    path.to_str().unwrap().to_string()
}

async fn get_contract_instance() -> (MyContract, ContractId) {
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
    .await;
    let wallet = wallets.pop().unwrap();

    let id = Contract::deploy(
        &get_path("out/debug/mock-oracle-contract.bin".to_string()),
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(get_path(
            "out/debug/mock-oracle-contract-storage_slots.json".to_string(),
        ))),
    )
    .await
    .unwrap();

    let instance = MyContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_get_contract_id() {
    let (instance, _id) = get_contract_instance().await;
    let new_price: u64 = 100;
    // Increment the counter
    let _result = instance
        .methods()
        .set_price(new_price)
        .call()
        .await
        .unwrap();

    // Get the current value of the counter
    let result = instance.methods().get_price().call().await.unwrap();

    // Check that the current value of the counter is 1.
    // Recall that the initial value of the counter was 0.
    assert_eq!(result.value, new_price);

    // Now you have an instance of your contract you can use to test each function
}
