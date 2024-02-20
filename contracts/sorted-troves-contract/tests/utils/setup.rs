use fuels::prelude::*;
use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;
use rand::{self, Rng};
use test_utils::interfaces::sorted_troves::{sorted_troves_abi::initialize, SortedTroves};
use test_utils::setup::common::{deploy_sorted_troves, get_absolute_path_from_relative};

abigen!(Contract(
    name = "MockTroveManagerContract",
    abi = "contracts/tests-artifacts-sorted-troves-contract/out/debug/mock-trove-manager-contract-abi.json"
));

const MOCK_TROVE_MANAGER_BINARY_PATH: &str =
    "contracts/tests-artifacts-sorted-troves-contract/out/debug/mock-trove-manager-contract.bin";

pub async fn deploy_mock_trove_manager_contract(
    wallet: &WalletUnlocked,
) -> MockTroveManagerContract<WalletUnlocked> {
    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();
    let tx_parms = TxPolicies::default().with_gas_price(1);

    let id = Contract::load_from(
        &get_absolute_path_from_relative(MOCK_TROVE_MANAGER_BINARY_PATH),
        LoadConfiguration::default().with_salt(salt),
    )
    .unwrap()
    .deploy(&wallet.clone(), tx_parms)
    .await
    .unwrap();

    MockTroveManagerContract::new(id, wallet.clone())
}

pub async fn set_nominal_icr_and_insert(
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    sorted_troves: &SortedTroves<WalletUnlocked>,
    new_id: Identity,
    new_icr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: AssetId,
) -> FuelCallResponse<()> {
    let tx_params = TxPolicies::default().with_gas_price(1);

    trove_manager
        .methods()
        .set_nominal_icr_and_insert(new_id, new_icr, prev_id, next_id, asset.into())
        .with_contracts(&[sorted_troves])
        .with_tx_policies(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn get_nominal_icr(
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
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
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    sorted_troves: &SortedTroves<WalletUnlocked>,
    id: Identity,
    asset: AssetId,
) -> FuelCallResponse<()> {
    let tx_params = TxPolicies::default().with_gas_price(1);

    trove_manager
        .methods()
        .remove(id, asset.into())
        .with_contracts(&[sorted_troves])
        .with_tx_policies(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn setup(
    num_wallets: Option<u64>,
) -> (
    SortedTroves<WalletUnlocked>,
    MockTroveManagerContract<WalletUnlocked>,
    WalletUnlocked,
    WalletUnlocked,
    Vec<WalletUnlocked>,
) {
    // Launch a local network and deploy the contract
    // let config = Config {
    //     manual_blocks_enabled: true, // Necessary so the `produce_blocks` API can be used locally
    //     ..Config::local_node()
    // };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            num_wallets,         /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();
    let wallet3 = wallets.pop().unwrap();
    let wallet4 = wallets.pop().unwrap();

    let st_instance = deploy_sorted_troves(&wallet).await;
    let trove_instance = deploy_mock_trove_manager_contract(&wallet2).await;

    (st_instance, trove_instance, wallet3, wallet4, wallets)
}

pub async fn initialize_st_and_tm(
    sorted_troves: &SortedTroves<WalletUnlocked>,
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    max_size: u64,
    asset: AssetId,
) {
    initialize(
        sorted_troves,
        max_size,
        trove_manager.contract_id().into(),
        trove_manager.contract_id().into(),
    )
    .await
    .unwrap();

    trove_manager
        .methods()
        .initialize(
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
            sorted_troves.contract_id(),
        )
        .call()
        .await
        .unwrap();

    trove_manager
        .methods()
        .add_asset(asset.into(), trove_manager.contract_id())
        .with_contracts(&[sorted_troves])
        .call()
        .await
        .unwrap();
}
