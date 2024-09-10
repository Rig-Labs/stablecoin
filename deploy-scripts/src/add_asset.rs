use crate::deploy::deployment::*;
use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::{Bits256, Identity, U256};
use pbr::ProgressBar;
use serde_json::json;
use std::{fs::File, io::Write};
use test_utils::data_structures::PRECISION;
use test_utils::interfaces::oracle::oracle_abi;
use test_utils::interfaces::protocol_manager::protocol_manager_abi;
use test_utils::interfaces::pyth_oracle::{
    pyth_oracle_abi, PythPrice, PythPriceFeed, PYTH_TIMESTAMP,
};
use test_utils::interfaces::redstone_oracle::{redstone_oracle_abi, redstone_price_feed_with_id};
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

pub async fn add_asset() {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let address = wallet.address();
    println!("🔑 Wallet address: {}", address);

    let core_contracts = load_core_contracts(wallet.clone());

    let asset_contracts = upload_asset(&wallet, &None).await;

    query_oracles(&asset_contracts).await;

    println!("Are you sure you want to initialize the asset? (y/n)");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return;
    }

    initialize_asset(&core_contracts, &asset_contracts, None).await;

    write_asset_contracts_to_file(vec![asset_contracts]);

    println!("Asset contracts added successfully");
}

pub async fn upload_asset(
    wallet: &WalletUnlocked,
    existing_contracts: &Option<ExistingAssetContracts>,
) -> AssetContracts<WalletUnlocked> {
    println!("Deploying asset contracts...");
    let mut pb = ProgressBar::new(6);
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

            let oracle = deploy_oracle(
                &wallet,
                contracts.pyth_oracle,
                contracts.pyth_precision,
                contracts.pyth_price_id,
                contracts.redstone_oracle,
                contracts.redstone_precision,
                contracts.redstone_price_id,
            )
            .await;

            return AssetContracts {
                oracle,
                mock_pyth_oracle: PythCore::new(contracts.pyth_oracle, wallet.clone()),
                mock_redstone_oracle: RedstoneCore::new(contracts.redstone_oracle, wallet.clone()),
                asset,
                trove_manager,
                asset_id,
                pyth_price_id: contracts.pyth_price_id,
                pyth_precision: contracts.pyth_precision,
                redstone_price_id: contracts.redstone_price_id,
                redstone_precision: contracts.redstone_precision,
            };
        }
        None => {
            let asset = deploy_token(&wallet).await;
            pb.inc();
            let asset_id: AssetId = asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into();

            let pyth_price_id = Bits256::from(asset_id);
            let redstone_price_id = U256::from(pyth_price_id.0);

            let pyth = deploy_mock_pyth_oracle(&wallet).await;
            let redstone = deploy_mock_redstone_oracle(&wallet).await;
            let oracle = deploy_oracle(
                &wallet,
                pyth.contract_id().into(),
                9,
                pyth_price_id,
                redstone.contract_id().into(),
                9,
                redstone_price_id,
            )
            .await;
            pb.inc();

            println!("Deploying asset contracts... Done");
            println!("Oracle: {}", oracle.contract_id());
            println!("Mock Pyth Oracle: {}", pyth.contract_id());
            println!("Mock Redstone Oracle: {}", redstone.contract_id());
            println!("Trove Manager: {}", trove_manager.contract_id());
            println!("Asset: {}", asset.contract_id());

            let _ = token_abi::initialize(
                &asset,
                1_000_000_000,
                &Identity::Address(wallet.address().into()),
                "MOCK".to_string(),
                "MOCK".to_string(),
            )
            .await;
            pb.inc();
            let pyth_feed = vec![(
                pyth_price_id,
                PythPriceFeed {
                    price: PythPrice {
                        price: 1 * PRECISION,
                        publish_time: PYTH_TIMESTAMP,
                    },
                },
            )];
            let redstone_feed = redstone_price_feed_with_id(redstone_price_id, vec![1]);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(&pyth, pyth_feed).await;
            pb.inc();

            redstone_oracle_abi::write_prices(&redstone, redstone_feed).await;
            redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
            pb.inc();

            return AssetContracts {
                oracle,
                mock_pyth_oracle: pyth,
                mock_redstone_oracle: redstone,
                trove_manager,
                asset,
                asset_id,
                pyth_price_id,
                pyth_precision: 9,
                redstone_price_id,
                redstone_precision: 9,
            };
        }
    }
}

pub async fn initialize_asset<T: Account>(
    core_protocol_contracts: &ProtocolContracts<T>,
    asset_contracts: &AssetContracts<T>,
    existing_asset_contracts: Option<ExistingAssetContracts>,
) -> () {
    println!("Initializing asset contracts...");
    let mut pb = ProgressBar::new(7);

    match existing_asset_contracts {
        Some(_) => {}
        None => {}
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
    pb.inc();
}

pub fn load_core_contracts(wallet: WalletUnlocked) -> ProtocolContracts<WalletUnlocked> {
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
        community_issuance,
        vesting_contract,
        coll_surplus_pool,
        sorted_troves,
        default_pool,
        active_pool,
    };

    protocol_contracts
}

fn write_asset_contracts_to_file(asset_contracts: Vec<AssetContracts<WalletUnlocked>>) {
    let mut file = File::create("asset_contracts.json").unwrap();
    // TODO try and add the pyth price id
    let json = json!({
        "asset_contracts": asset_contracts.iter().map(|asset_contract| {
            json!({
                "oracle": asset_contract.oracle.contract_id().to_string(),
                "trove_manager": asset_contract.trove_manager.contract_id().to_string(),
                "asset_contract": asset_contract.asset.contract_id().to_string(),
                "asset_id": asset_contract.asset_id.to_string(),
                "pyth_precision": asset_contract.pyth_precision,
                "pyth_contract": asset_contract.mock_pyth_oracle.contract_id().to_string(),
                "redstone_contract": asset_contract.mock_redstone_oracle.contract_id().to_string(),
                "redstone_price_id": asset_contract.redstone_price_id.to_string(),
                "redstone_precision": asset_contract.redstone_precision,
            })
        }).collect::<Vec<serde_json::Value>>()
    });

    file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
        .unwrap();
}

async fn query_oracles(asset_contracts: &AssetContracts<WalletUnlocked>) {
    let current_price = oracle_abi::get_price(
        &asset_contracts.oracle,
        &asset_contracts.mock_pyth_oracle,
        &asset_contracts.mock_redstone_oracle,
    )
    .await
    .value;

    let current_pyth_price = pyth_oracle_abi::price(
        &asset_contracts.mock_pyth_oracle,
        &asset_contracts.pyth_price_id,
    )
    .await
    .value
    .price;

    let current_redstone_price = redstone_oracle_abi::read_prices(
        &asset_contracts.mock_redstone_oracle,
        vec![asset_contracts.redstone_price_id],
    )
    .await
    .value[0]
        .as_u64();

    println!(
        "Current oracle proxy price: {:.9}",
        current_price as f64 / 1_000_000_000.0
    );
    println!(
        "Current pyth price: {:.9}",
        current_pyth_price as f64 / 1_000_000_000.0
    );
    println!(
        "Current redstone price: {:.9}",
        current_redstone_price as f64 / 1_000_000_000.0
    );
}
