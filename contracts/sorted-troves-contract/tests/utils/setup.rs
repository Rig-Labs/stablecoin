use fuels::prelude::*;

use super::trove_manager::deploy_trove_manager_contract;
// TODO: do setup instead of copy/pasted code with minor adjustments

// Load abi from json
abigen!(
    Contract(
        name = "SortedTroves",
        abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
    ),
    Contract(
        name = "TroveManagerContract",
        abi = "contracts/trove-manager-contract/out/debug/trove-manager-contract-abi.json"
    )
);

fn get_path(sub_path: String) -> String {
    let mut path = std::env::current_dir().unwrap();
    path.push(sub_path);
    path.to_str().unwrap().to_string()
}
pub async fn setup() -> (
    SortedTroves,
    TroveManagerContract,
    WalletUnlocked,
    WalletUnlocked,
) {
    // Launch a local network and deploy the contract
    let config = Config {
        manual_blocks_enabled: true, // Necessary so the `produce_blocks` API can be used locally
        ..Config::local_node()
    };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(4),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        Some(config),
        None,
    )
    .await;

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();
    let wallet3 = wallets.pop().unwrap();
    let wallet4 = wallets.pop().unwrap();

    let id = Contract::deploy(
        &get_path("out/debug/sorted-troves-contract.bin".to_string()),
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(get_path(
            "out/debug/sorted-troves-contract-storage_slots.json".to_string(),
        ))),
    )
    .await
    .unwrap();

    let st_instance = SortedTroves::new(id.clone(), wallet);

    let trove_instance = deploy_trove_manager_contract(&wallet2).await;

    (st_instance, trove_instance, wallet3, wallet4)
}

pub async fn initialize(
    sorted_troves: &SortedTroves,
    trove_manager: &TroveManagerContract,
    max_size: u64,
) {
    let _result = sorted_troves
        .methods()
        .set_params(
            max_size,
            trove_manager.contract_id().into(),
            trove_manager.contract_id().into(),
        )
        .call()
        .await
        .unwrap();

    let _result = trove_manager
        .methods()
        .initialize(sorted_troves.contract_id().into())
        .call()
        .await
        .unwrap();
}
