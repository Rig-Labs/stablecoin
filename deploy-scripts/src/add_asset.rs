use crate::constants;
use crate::utils::utils::*;
use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::{Bits256, U256};
use serde_json::json;

use std::str::FromStr;
use std::{fs::File, io::Write};
use test_utils::data_structures::{
    AssetConfig, AssetContracts, ExistingAssetContracts, PythConfig, RedstoneConfig,
};
use test_utils::interfaces::oracle::oracle_abi;
use test_utils::interfaces::pyth_oracle::pyth_oracle_abi;
use test_utils::interfaces::redstone_oracle::redstone_oracle_abi;

use test_utils::setup::common::*;

pub async fn add_asset() {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let address: Address = wallet.address().into();
    println!("🔑 Wallet address: {}", address);

    let core_contracts = load_core_contracts(wallet.clone());
    // you will need to set the existing asset contracts here manually and uncomment the below line
    let mut existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        asset: Some(AssetConfig {
            asset: ContractId::from_str(constants::TESTNET_ASSET_CONTRACT_ID).unwrap(),
            asset_id: AssetId::from_str(constants::TESTNET_ETH_ASSET_ID).unwrap(),
            fuel_vm_decimals: 9,
        }),
        pyth_oracle: Some(PythConfig {
            contract: ContractId::from_str(constants::PYTH_TESTNET_CONTRACT_ID).unwrap(),
            price_id: Bits256::from_hex_str(constants::TESTNET_PYTH_ETH_PRICE_ID).unwrap(),
        }),
        redstone_oracle: None,
    };

    existing_asset_to_initialize.asset = None;

    if existing_asset_to_initialize.asset.is_none()
        || existing_asset_to_initialize.pyth_oracle.is_none()
        || existing_asset_to_initialize.redstone_oracle.is_none()
    {
        // if rpc url doesn't have testnet in it then cause a failure so it's obvious
        let rpc_url = std::env::var("RPC").unwrap();
        if !rpc_url.contains("testnet") {
            panic!("RPC URL does not contain testnet, make sure you set the correct RPC URL in the .env file");
        }
        println!("Initializing new asset");
    } else {
        println!("Existing asset to initialize");
    }

    // Deploy the asset contracts
    let asset_contracts =
        deploy_asset_contracts(&wallet, &existing_asset_to_initialize, false).await;

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

    initialize_asset(&core_contracts, &asset_contracts)
        .await
        .unwrap();

    write_asset_contracts_to_file(vec![asset_contracts]);

    println!("Asset contracts added successfully");
}

fn write_asset_contracts_to_file(asset_contracts: Vec<AssetContracts<WalletUnlocked>>) {
    // Read existing contracts.json
    let mut contracts: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("contracts.json").expect("Failed to read contracts.json"),
    )
    .expect("Failed to parse contracts.json");

    // Get existing asset_contracts or create an empty array if it doesn't exist
    let mut existing_asset_contracts = contracts["asset_contracts"]
        .as_array()
        .cloned()
        .unwrap_or_else(Vec::new);

    // Add new asset_contracts to the existing ones
    for asset_contract in asset_contracts {
        existing_asset_contracts.push(json!({
            "oracle": asset_contract.oracle.contract_id().to_string(),
            "trove_manager": asset_contract.trove_manager.contract_id().to_string(),
            "asset_contract": asset_contract.asset.contract_id().to_string(),
            "asset_id": format!("0x{}", asset_contract.asset_id.to_string()),
            "pyth_price_id": to_hex_str(&asset_contract.pyth_price_id),
            "pyth_contract": asset_contract.mock_pyth_oracle.contract_id().to_string(),
            "redstone_contract": asset_contract.mock_redstone_oracle.contract_id().to_string(),
            "redstone_price_id": asset_contract.redstone_price_id.to_string(),
            "redstone_precision": asset_contract.redstone_precision,
            "fuel_vm_decimals": asset_contract.fuel_vm_decimals,
        }));
    }

    // Update asset_contracts field with the combined list
    contracts["asset_contracts"] = json!(existing_asset_contracts);

    // Write updated contracts back to file
    let mut file =
        File::create("contracts.json").expect("Failed to open contracts.json for writing");
    file.write_all(serde_json::to_string_pretty(&contracts).unwrap().as_bytes())
        .expect("Failed to write to contracts.json");
}

async fn query_oracles(asset_contracts: &AssetContracts<WalletUnlocked>) {
    let current_pyth_price = pyth_oracle_abi::price_unsafe(
        &asset_contracts.mock_pyth_oracle,
        &asset_contracts.pyth_price_id,
    )
    .await
    .value;

    let pyth_precision = current_pyth_price.exponent as usize;
    println!(
        "Current pyth price: {:.precision$}",
        current_pyth_price.price as f64 / 10f64.powi(pyth_precision.try_into().unwrap()),
        precision = pyth_precision
    );
    let current_price = oracle_abi::get_price(
        &asset_contracts.oracle,
        &asset_contracts.mock_pyth_oracle,
        &Some(asset_contracts.mock_redstone_oracle.clone()),
    )
    .await
    .value;

    println!(
        "Current oracle proxy price: {:.9}",
        current_price as f64 / 1_000_000_000.0
    );

    let redstone_precision = asset_contracts.redstone_precision as usize;
    let current_redstone_price = redstone_oracle_abi::read_prices(
        &asset_contracts.mock_redstone_oracle,
        vec![asset_contracts.redstone_price_id],
    )
    .await
    .value[0]
        .as_u64();

    println!(
        "Current redstone price: {:.precision$}",
        current_redstone_price as f64 / 10f64.powi(redstone_precision.try_into().unwrap()),
        precision = redstone_precision
    );
}
pub fn to_hex_str(bits: &Bits256) -> String {
    format!("0x{}", hex::encode(bits.0))
}
