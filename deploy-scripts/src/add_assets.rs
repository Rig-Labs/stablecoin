use crate::deploy::deployment::*;
use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::{Bits256, Identity};
use pbr::ProgressBar;
use serde_json::json;
use std::{fs::File, io::Write};
use test_utils::data_structures::PRECISION;
use test_utils::interfaces::protocol_manager::protocol_manager_abi;
use test_utils::interfaces::pyth_oracle::{pyth_oracle_abi, PythPrice, PythPriceFeed};
use test_utils::interfaces::token::token_abi;
use test_utils::interfaces::trove_manager::trove_manager_abi;
use test_utils::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, community_issuance::CommunityIssuance,
    default_pool::DefaultPool, fpt_staking::FPTStaking, fpt_token::FPTToken, oracle::Oracle,
    protocol_manager::ProtocolManager, pyth_oracle::PythCore, redstone_oracle::RedstoneCore,
    sorted_troves::SortedTroves, stability_pool::StabilityPool, token::Token,
    trove_manager::TroveManagerContract, usdf_token::USDFToken, vesting::VestingContract,
};
use test_utils::setup::common::*;

pub async fn add_assets() {
    dotenv().ok();

    let rpc = std::env::var("RPC").expect("❌ Cannot find RPC in .env file");
    println!("RPC: {}", rpc);

    let wallet = setup_wallet(&rpc).await;
    let address = wallet.address();
    println!("🔑 Wallet address: {}", address);

    let core_contracts = load_core_contracts(wallet.clone());

    let asset_contracts = upload_asset(wallet.clone(), &None).await;

    initialize_asset(wallet.clone(), &core_contracts, &asset_contracts, None).await;

    write_asset_contracts_to_file(vec![asset_contracts]);

    println!("Asset contracts");
}

fn load_core_contracts(wallet: WalletUnlocked) -> ProtocolContracts<WalletUnlocked> {
    let json = std::fs::read_to_string("contracts.json").unwrap();
    let contracts: serde_json::Value = serde_json::from_str(&json).unwrap();

    let borrow_operations_contract_id: Bech32ContractId = contracts["borrow_operations"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let borrow_operations = BorrowOperations::new(borrow_operations_contract_id, wallet.clone());

    let usdf_contract_id: Bech32ContractId = contracts["usdf"].as_str().unwrap().parse().unwrap();
    let usdf = test_utils::interfaces::usdf_token::USDFToken::new(usdf_contract_id, wallet.clone());

    let stability_pool_contract_id: Bech32ContractId = contracts["stability_pool"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let stability_pool = StabilityPool::new(stability_pool_contract_id, wallet.clone());

    let protocol_manager_contract_id: Bech32ContractId = contracts["protocol_manager"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let protocol_manager = ProtocolManager::new(protocol_manager_contract_id, wallet.clone());

    let fpt_staking_contract_id: Bech32ContractId =
        contracts["fpt_staking"].as_str().unwrap().parse().unwrap();
    let fpt_staking = FPTStaking::new(fpt_staking_contract_id, wallet.clone());

    let fpt_token_contract_id: Bech32ContractId =
        contracts["fpt_token"].as_str().unwrap().parse().unwrap();
    let fpt_token = FPTToken::new(fpt_token_contract_id.clone(), wallet.clone());

    let community_issuance_contract_id: Bech32ContractId = contracts["community_issuance"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let community_issuance = CommunityIssuance::new(community_issuance_contract_id, wallet.clone());

    let coll_surplus_pool_contract_id: Bech32ContractId = contracts["coll_surplus_pool"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let coll_surplus_pool = CollSurplusPool::new(coll_surplus_pool_contract_id, wallet.clone());

    let default_pool_contract_id: Bech32ContractId =
        contracts["default_pool"].as_str().unwrap().parse().unwrap();
    let default_pool = DefaultPool::new(default_pool_contract_id, wallet.clone());

    let active_pool_contract_id: Bech32ContractId =
        contracts["active_pool"].as_str().unwrap().parse().unwrap();
    let active_pool = ActivePool::new(active_pool_contract_id, wallet.clone());

    let sorted_troves_contract_id: Bech32ContractId = contracts["sorted_troves"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let sorted_troves = SortedTroves::new(sorted_troves_contract_id, wallet.clone());

    let vesting_contract_id: Bech32ContractId = contracts["vesting_contract"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let vesting_contract = VestingContract::new(vesting_contract_id, wallet.clone());

    // TODO: remove this since it's redundant with fpt token
    let fpt = Token::new(fpt_token_contract_id.clone(), wallet.clone());

    let asset_contracts = vec![];

    let protocol_contracts = ProtocolContracts {
        borrow_operations,
        usdf,
        stability_pool,
        protocol_manager,
        asset_contracts,
        fpt_staking,
        fpt_token,
        fpt,
        community_issuance,
        vesting_contract,
        coll_surplus_pool,
        sorted_troves,
        default_pool,
        active_pool,
    };

    protocol_contracts
}

async fn deploy_and_initialize_assets(
    wallet: WalletUnlocked,
    core_contracts: ProtocolContracts<WalletUnlocked>,
) -> Vec<AssetContracts<WalletUnlocked>> {
    let mut asset_contracts = Vec::new();

    // // Deploy and initialize FUEL asset
    // let fuel_asset = deploy_and_initialize_asset(
    //     wallet.clone(),
    //     &core_contracts,
    //     "Fuel".to_string(),
    //     "FUEL".to_string(),
    //     None,
    // )
    // .await;
    // asset_contracts.push(fuel_asset);

    // // Deploy and initialize stFUEL asset
    // let stfuel_asset = deploy_and_initialize_asset(
    //     wallet.clone(),
    //     &core_contracts,
    //     "stFuel".to_string(),
    //     "stFUEL".to_string(),
    //     None,
    // )
    // .await;
    // asset_contracts.push(stfuel_asset);

    asset_contracts
}

fn write_asset_contracts_to_file(asset_contracts: Vec<AssetContracts<WalletUnlocked>>) {
    let mut file = File::create("asset_contracts.json").unwrap();

    let json = json!({
        "asset_contracts": asset_contracts.iter().map(|asset_contract| {
            json!({
                "oracle": asset_contract.oracle.contract_id().to_string(),
                "trove_manager": asset_contract.trove_manager.contract_id().to_string(),
                "asset_contract": asset_contract.asset.contract_id().to_string(),
                "asset_id": asset_contract.asset_id.to_string(),
            })
        }).collect::<Vec<serde_json::Value>>()
    });

    file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
        .unwrap();
}

pub async fn upload_asset(
    wallet: WalletUnlocked,
    existing_contracts: &Option<ExistingAssetContracts>,
) -> AssetContracts<WalletUnlocked> {
    println!("Deploying asset contracts...");
    let mut pb = ProgressBar::new(3);
    let trove_manager = deploy_trove_manager_contract(&wallet).await;
    pb.inc();

    match existing_contracts {
        Some(contracts) => {
            pb.finish();
            let asset = Token::new(contracts.asset, wallet.clone());
            let asset_id: AssetId = asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into();

            return AssetContracts {
                oracle: Oracle::new(contracts.oracle, wallet.clone()),
                mock_pyth_oracle: PythCore::new(contracts.pyth_oracle, wallet.clone()),
                mock_redstone_oracle: RedstoneCore::new(contracts.redstone_oracle, wallet.clone()),
                asset,
                trove_manager,
                asset_id,
            };
        }
        None => {
            let pyth = deploy_mock_pyth_oracle(&wallet).await;
            let redstone = deploy_mock_redstone_oracle(&wallet).await;
            let oracle = deploy_oracle(
                &wallet,
                pyth.contract_id().into(),
                9,
                redstone.contract_id().into(),
                9,
            )
            .await;
            pb.inc();
            let asset = deploy_token(&wallet).await;
            pb.inc();

            let asset_id: AssetId = asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into();

            println!("Deploying asset contracts... Done");
            println!("Oracle: {}", oracle.contract_id());
            println!("Mock Pyth Oracle: {}", pyth.contract_id());
            println!("Mock Redstone Oracle: {}", redstone.contract_id());
            println!("Trove Manager: {}", trove_manager.contract_id());
            println!("Asset: {}", asset.contract_id());

            return AssetContracts {
                oracle,
                mock_pyth_oracle: pyth,
                mock_redstone_oracle: redstone,
                trove_manager,
                asset,
                asset_id,
            };
        }
    }
}

pub async fn initialize_asset<T: Account>(
    wallet: WalletUnlocked,
    core_protocol_contracts: &ProtocolContracts<T>,
    asset_contracts: &AssetContracts<T>,
    existing_asset_contracts: Option<ExistingAssetContracts>,
) -> () {
    println!("Initializing asset contracts...");
    let mut pb = ProgressBar::new(7);

    match existing_asset_contracts {
        Some(_) => {}
        None => {
            let _ = token_abi::initialize(
                &asset_contracts.asset,
                1_000_000_000,
                &Identity::Address(wallet.address().into()),
                "MOCK".to_string(),
                "MOCK".to_string(),
            )
            .await;
            wait();
            pb.inc();

            let pyth_feed = vec![(
                Bits256::zeroed(),
                PythPriceFeed {
                    price: PythPrice {
                        price: 1_000 * PRECISION,
                        publish_time: 1,
                    },
                },
            )];

            pyth_oracle_abi::update_price_feeds(&asset_contracts.mock_pyth_oracle, pyth_feed).await;
            wait();
            pb.inc();
        }
    }

    let _ = trove_manager_abi::initialize(
        &asset_contracts.trove_manager,
        core_protocol_contracts
            .borrow_operations
            .contract_id()
            .into(),
        core_protocol_contracts.sorted_troves.contract_id().into(),
        asset_contracts.oracle.contract_id().into(),
        core_protocol_contracts.stability_pool.contract_id().into(),
        core_protocol_contracts.default_pool.contract_id().into(),
        core_protocol_contracts.active_pool.contract_id().into(),
        core_protocol_contracts
            .coll_surplus_pool
            .contract_id()
            .into(),
        core_protocol_contracts.usdf.contract_id().into(),
        asset_contracts.asset_id,
        core_protocol_contracts
            .protocol_manager
            .contract_id()
            .into(),
    )
    .await;
    wait();
    pb.inc();

    let _ = protocol_manager_abi::register_asset(
        &core_protocol_contracts.protocol_manager,
        asset_contracts.asset_id,
        asset_contracts.trove_manager.contract_id().into(),
        asset_contracts.oracle.contract_id().into(),
        &core_protocol_contracts.borrow_operations,
        &core_protocol_contracts.stability_pool,
        &core_protocol_contracts.usdf,
        &core_protocol_contracts.fpt_staking,
        &core_protocol_contracts.coll_surplus_pool,
        &core_protocol_contracts.default_pool,
        &core_protocol_contracts.active_pool,
        &core_protocol_contracts.sorted_troves,
    )
    .await;
    wait();
    pb.inc();
}

pub fn wait() {
    // Necessary for random instances where the 'UTXO' cannot be found
    std::thread::sleep(std::time::Duration::from_secs(15));
}
