use fuels::prelude::*;

use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;
use test_utils::interfaces::sorted_troves::{sorted_troves_abi::initialize, SortedTroves};

use test_utils::setup::common::deploy_sorted_troves;
// TODO: do setup instead of copy/pasted code with minor adjustments

abigen!(Contract(
    name = "MockTroveManagerContract",
    abi = "contracts/sorted-troves-contract/tests/artifacts/out/debug/mock-trove-manager-contract-abi.json"
));

const MOCK_TROVE_MANAGER_BINARY_PATH: &str =
    "tests/artifacts/out/debug/mock-trove-manager-contract.bin";
const MOCK_TROVE_MANAGER_CONTRACT_STORAGE_PATH: &str =
    "tests/artifacts/out/debug/mock-trove-manager-contract-storage_slots.json";

pub fn get_relative_path(path: String) -> String {
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join(path);
    return path.to_str().unwrap().to_string();
}

pub async fn deploy_mock_trove_manager_contract(
    wallet: &WalletUnlocked,
) -> MockTroveManagerContract {
    let id = Contract::deploy(
        &get_relative_path(MOCK_TROVE_MANAGER_BINARY_PATH.to_string()),
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(get_relative_path(
            MOCK_TROVE_MANAGER_CONTRACT_STORAGE_PATH.to_string(),
        ))),
    )
    .await
    .unwrap();

    MockTroveManagerContract::new(id, wallet.clone())
}

pub async fn set_nominal_icr_and_insert(
    trove_manager: &MockTroveManagerContract,
    sorted_troves: &SortedTroves,
    new_id: Identity,
    new_icr: u64,
    prev_id: Identity,
    next_id: Identity,
) -> FuelCallResponse<()> {
    let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

    trove_manager
        .methods()
        .set_nominal_icr_and_insert(new_id, new_icr, prev_id, next_id)
        .set_contracts(&[sorted_troves])
        .tx_params(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn get_nominal_icr(
    trove_manager: &MockTroveManagerContract,
    id: Identity,
) -> FuelCallResponse<u64> {
    trove_manager
        .methods()
        .get_nominal_icr(id)
        .call()
        .await
        .unwrap()
}

pub async fn remove(
    trove_manager: &MockTroveManagerContract,
    sorted_troves: &SortedTroves,
    id: Identity,
) -> FuelCallResponse<()> {
    let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

    trove_manager
        .methods()
        .remove(id)
        .set_contracts(&[sorted_troves])
        .tx_params(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn setup(
    num_wallets: Option<u64>,
) -> (
    SortedTroves,
    MockTroveManagerContract,
    WalletUnlocked,
    WalletUnlocked,
    Vec<WalletUnlocked>,
) {
    // Launch a local network and deploy the contract
    let config = Config {
        manual_blocks_enabled: true, // Necessary so the `produce_blocks` API can be used locally
        ..Config::local_node()
    };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            num_wallets,         /* Single wallet */
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

    let st_instance = deploy_sorted_troves(&wallet).await;
    let trove_instance = deploy_mock_trove_manager_contract(&wallet2).await;

    (st_instance, trove_instance, wallet3, wallet4, wallets)
}

pub async fn initialize_st_and_tm(
    sorted_troves: &SortedTroves,
    trove_manager: &MockTroveManagerContract,
    max_size: u64,
) {
    let _ = initialize(
        sorted_troves,
        max_size,
        trove_manager.contract_id().into(),
        trove_manager.contract_id().into(),
    )
    .await;

    trove_manager
        .methods()
        .initialize(
            sorted_troves.contract_id().into(),
            sorted_troves.contract_id().into(),
            sorted_troves.contract_id().into(),
            sorted_troves.contract_id().into(),
            sorted_troves.contract_id().into(),
            sorted_troves.contract_id().into(),
        )
        .call()
        .await
        .unwrap();
}
