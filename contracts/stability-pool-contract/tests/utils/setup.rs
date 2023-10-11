use fuels::accounts::fuel_crypto::rand::{self, Rng};
use fuels::prelude::*;

use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;
use test_utils::interfaces::sorted_troves::SortedTroves;
use test_utils::interfaces::stability_pool::{stability_pool_abi, StabilityPool};
use test_utils::interfaces::token::{token_abi, Token};
use test_utils::setup::common::{
    deploy_active_pool, deploy_stability_pool, deploy_token, get_absolute_path_from_relative,
};

abigen!(Contract(
    name = "MockTroveManagerContract",
    abi = "contracts/tests-artifacts-stability-pool-contract/out/debug/mock-trove-manager-sp-contract-abi.json"
));

const MOCK_TROVE_MANAGER_BINARY_PATH: &str =
    "contracts/tests-artifacts-stability-pool-contract/out/debug/mock-trove-manager-sp-contract.bin";

pub fn get_relative_path(path: String) -> String {
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join(path);
    return path.to_str().unwrap().to_string();
}

pub async fn deploy_mock_trove_manager_contract(
    wallet: &WalletUnlocked,
) -> MockTroveManagerContract<WalletUnlocked> {
    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();
    let tx_parms = TxParameters::default().with_gas_price(1);

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
    let tx_params = TxParameters::default().with_gas_price(1);

    trove_manager
        .methods()
        .set_nominal_icr_and_insert(new_id, new_icr, prev_id, next_id, asset.into())
        .with_contracts(&[sorted_troves])
        .tx_params(tx_params)
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

pub async fn offset(
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    stability_pool: &StabilityPool<WalletUnlocked>,
    fuel_token: &Token<WalletUnlocked>,
    usdf_token: &Token<WalletUnlocked>,
    coll_to_offset: u64,
    debt_to_offset: u64,
) -> FuelCallResponse<()> {
    let tx_params = TxParameters::default().with_gas_price(1);

    trove_manager
        .methods()
        .offset(coll_to_offset, debt_to_offset)
        .with_contracts(&[stability_pool, fuel_token, usdf_token])
        .tx_params(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn initialize(
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    borrow_operations: ContractId,
    sorted_troves: ContractId,
    stability_pool: ContractId,
) -> Result<FuelCallResponse<()>> {
    let tx_params = TxParameters::default().with_gas_price(1);

    trove_manager
        .methods()
        .initialize(borrow_operations, sorted_troves, stability_pool)
        .tx_params(tx_params)
        .call()
        .await
}

pub async fn remove(
    trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    sorted_troves: &SortedTroves<WalletUnlocked>,
    id: Identity,
    asset: AssetId,
) -> FuelCallResponse<()> {
    let tx_params = TxParameters::default().with_gas_price(1);

    trove_manager
        .methods()
        .remove(id, asset.into())
        .with_contracts(&[sorted_troves])
        .tx_params(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn setup(
    num_wallets: Option<u64>,
) -> (
    StabilityPool<WalletUnlocked>,
    MockTroveManagerContract<WalletUnlocked>,
    Token<WalletUnlocked>,
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

    let stability_pool = deploy_stability_pool(&wallet).await;
    let trove_instance = deploy_mock_trove_manager_contract(&wallet2).await;

    let fuel_token = deploy_token(&wallet).await;
    let usdf_token = deploy_token(&wallet).await;

    let active_pool = deploy_active_pool(&wallet).await;

    token_abi::initialize(
        &fuel_token,
        0,
        &Identity::Address(wallet.address().into()),
        "Fuel".to_string(),
        "FUEL".to_string(),
    )
    .await
    .unwrap();

    token_abi::initialize(
        &usdf_token,
        0,
        &Identity::Address(wallet.address().into()),
        "USDF".to_string(),
        "USDF".to_string(),
    )
    .await
    .unwrap();

    stability_pool_abi::initialize(
        &stability_pool,
        usdf_token.contract_id().into(),
        stability_pool.contract_id().into(),
        fuel_token.contract_id().into(),
        active_pool.contract_id().into(),
    )
    .await
    .unwrap();

    initialize(
        &trove_instance,
        stability_pool.contract_id().into(),
        stability_pool.contract_id().into(),
        stability_pool.contract_id().into(),
    )
    .await
    .unwrap();

    (
        stability_pool,
        trove_instance,
        fuel_token,
        wallet3,
        wallet4,
        wallets,
    )
}
