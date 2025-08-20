use fuels::prelude::*;
use rand::{self, Rng};

use fuels::programs::responses::CallResponse;
use fuels::types::Identity;
use test_utils::data_structures::ContractInstance;
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
    wallet: &Wallet,
) -> MockTroveManagerContract<Wallet> {
    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();
    let tx_parms = TxPolicies::default().with_tip(1);

    let id = Contract::load_from(
        &get_absolute_path_from_relative(MOCK_TROVE_MANAGER_BINARY_PATH),
        LoadConfiguration::default().with_salt(salt),
    )
    .unwrap()
    .deploy(&wallet.clone(), tx_parms)
    .await
    .unwrap()
    .contract_id;

    MockTroveManagerContract::new(id, wallet.clone())
}

pub async fn set_nominal_icr_and_insert(
    trove_manager: &MockTroveManagerContract<Wallet>,
    sorted_troves: &SortedTroves<Wallet>,
    new_id: Identity,
    new_icr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: AssetId,
) -> CallResponse<()> {
    let tx_params = TxPolicies::default().with_tip(1);

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
    trove_manager: &MockTroveManagerContract<Wallet>,
    id: Identity,
) -> CallResponse<u64> {
    trove_manager
        .methods()
        .get_nominal_icr(id)
        .call()
        .await
        .unwrap()
}

pub async fn offset(
    trove_manager: &MockTroveManagerContract<Wallet>,
    stability_pool: &StabilityPool<Wallet>,
    mock_token: &Token<Wallet>,
    usdm_token: &Token<Wallet>,
    coll_to_offset: u64,
    debt_to_offset: u64,
) -> CallResponse<()> {
    let tx_params = TxPolicies::default().with_tip(1);

    trove_manager
        .methods()
        .offset(coll_to_offset, debt_to_offset)
        .with_contracts(&[stability_pool, mock_token, usdm_token])
        .with_tx_policies(tx_params)
        .call()
        .await
        .unwrap()
}

pub async fn initialize(
    trove_manager: &MockTroveManagerContract<Wallet>,
    borrow_operations: ContractId,
    sorted_troves: ContractId,
    stability_pool: ContractId,
) -> Result<CallResponse<()>> {
    let tx_params = TxPolicies::default().with_tip(1);

    trove_manager
        .methods()
        .initialize(borrow_operations, sorted_troves, stability_pool)
        .with_tx_policies(tx_params)
        .call()
        .await
}

pub async fn remove(
    trove_manager: &MockTroveManagerContract<Wallet>,
    sorted_troves: &SortedTroves<Wallet>,
    id: Identity,
    asset: AssetId,
) -> CallResponse<()> {
    let tx_params = TxPolicies::default().with_tip(1);

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
    ContractInstance<StabilityPool<Wallet>>,
    MockTroveManagerContract<Wallet>,
    Token<Wallet>,
    Wallet,
    Wallet,
    Vec<Wallet>,
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

    let stability_pool = deploy_stability_pool(&wallet).await;
    let sorted_troves = deploy_mock_trove_manager_contract(&wallet).await;
    let trove_instance = deploy_mock_trove_manager_contract(&wallet2).await;

    let mock_token = deploy_token(&wallet).await;
    let usdm_token = deploy_token(&wallet).await;

    let active_pool = deploy_active_pool(&wallet).await;

    token_abi::initialize(
        &mock_token,
        0,
        &Identity::Address(wallet.address().into()),
        "Mock".to_string(),
        "MOCK".to_string(),
    )
    .await
    .unwrap();

    token_abi::initialize(
        &usdm_token,
        0,
        &Identity::Address(wallet.address().into()),
        "USDM".to_string(),
        "USDM".to_string(),
    )
    .await
    .unwrap();

    stability_pool_abi::initialize(
        &stability_pool,
        usdm_token.contract_id().into(),
        stability_pool.contract.contract_id().into(),
        mock_token.contract_id().into(),
        active_pool.contract.contract_id().into(),
        sorted_troves.contract_id().into(),
    )
    .await
    .unwrap();

    initialize(
        &trove_instance,
        stability_pool.contract.contract_id().into(),
        stability_pool.contract.contract_id().into(),
        stability_pool.contract.contract_id().into(),
    )
    .await
    .unwrap();

    (
        stability_pool,
        trove_instance,
        mock_token,
        wallet3,
        wallet4,
        wallets,
    )
}
