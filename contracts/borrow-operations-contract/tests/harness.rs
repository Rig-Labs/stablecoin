use fuels::{prelude::*, tx::ContractId, types::Identity};

// Load abi from json
abigen!(Contract(
    name = "TroveManagerContract",
    abi = "contracts/borrow-operations-contract/out/debug/borrow-operations-contract-abi.json"
));

// get path
fn get_path(sub_path: String) -> String {
    let mut path = std::env::current_dir().unwrap();
    path.push(sub_path);
    path.to_str().unwrap().to_string()
}

async fn get_contract_instance() -> (TroveManagerContract, ContractId, WalletUnlocked) {
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

    let id = Contract::deploy(
        &get_path("out/debug/borrow-operations-contract.bin".to_string()),
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(get_path(
            "out/debug/borrow-operations-contract-storage_slots.json".to_string(),
        ))),
    )
    .await
    .unwrap();

    let instance = TroveManagerContract::new(id.clone(), wallet);

    (instance, id.into(), wallets[0].clone())
}

#[tokio::test]
async fn can_set_and_retrieve_irc() {
    let (instance, _id, admin) = get_contract_instance().await;
    let irc: u64 = 100;
    // Increment the counter
    let _result = instance
        .methods()
        .set_nominal_icr(Identity::Address(admin.address().into()), irc)
        .call()
        .await
        .unwrap();

    // Get the current value of the counter
    let result = instance
        .methods()
        .get_nominal_icr(Identity::Address(admin.address().into()))
        .call()
        .await
        .unwrap();

    // Check that the current value of the counter is 1.
    // Recall that the initial value of the counter was 0.
    assert_eq!(result.value, irc);

    // Now you have an instance of your contract you can use to test each function
}
