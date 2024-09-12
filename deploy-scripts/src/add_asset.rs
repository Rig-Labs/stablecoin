use crate::deploy::deployment::*;
use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::{Bits256, U256};
use serde_json::json;
use std::str::FromStr;
use std::{fs::File, io::Write};
use test_utils::data_structures::{AssetContracts, ExistingAssetContracts, ProtocolContracts};
use test_utils::interfaces::oracle::oracle_abi;
use test_utils::interfaces::pyth_oracle::pyth_oracle_abi;
use test_utils::interfaces::redstone_oracle::redstone_oracle_abi;

use test_utils::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, community_issuance::CommunityIssuance,
    default_pool::DefaultPool, fpt_staking::FPTStaking, fpt_token::FPTToken,
    protocol_manager::ProtocolManager, sorted_troves::SortedTroves, stability_pool::StabilityPool,
    vesting::VestingContract,
};
use test_utils::setup::common::*;

pub async fn add_asset() {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let address = wallet.address();
    println!("🔑 Wallet address: {}", address);

    let core_contracts = load_core_contracts(wallet.clone());

    // you will need to set the existing asset contracts here manually and uncomment the below line
    let mut existing_asset_to_initialize: Option<ExistingAssetContracts> =
        Some(ExistingAssetContracts {
            asset: ContractId::zeroed(),
            asset_id: AssetId::zeroed(),
            pyth_oracle: ContractId::zeroed(),
            pyth_precision: 9,
            pyth_price_id: Bits256::zeroed(),
            redstone_oracle: ContractId::zeroed(),
            redstone_price_id: U256::zero(),
            redstone_precision: 9,
        });
    existing_asset_to_initialize = None;

    match existing_asset_to_initialize {
        Some(_) => {
            println!("Existing asset to initialize");
        }
        None => {
            println!("Initializing new asset");
        }
    }
    // Deploy the asset contracts
    let asset_contracts = deploy_asset_contracts(&wallet, &existing_asset_to_initialize).await;

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

    initialize_asset(&core_contracts, &asset_contracts).await;

    write_asset_contracts_to_file(vec![asset_contracts]);

    println!("Asset contracts added successfully");
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

    let fpt_asset_id: AssetId =
        AssetId::from_str(contracts["fpt_asset_id"].as_str().unwrap()).unwrap();

    let asset_contracts = vec![];

    let protocol_contracts = ProtocolContracts {
        borrow_operations,
        usdf,
        stability_pool,
        protocol_manager,
        asset_contracts,
        fpt_staking,
        fpt_token,
        fpt_asset_id,
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
