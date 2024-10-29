use crate::constants::{self, MAINNET_CONTRACTS_FILE, TESTNET_CONTRACTS_FILE};
use crate::utils::utils::*;
use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::Bits256;
use serde_json::json;

use std::str::FromStr;
use std::{fs::File, io::Write};
use test_utils::data_structures::{
    AssetConfig, AssetContractsOptionalRedstone, ExistingAssetContracts, PythConfig,
};
use test_utils::interfaces::oracle::oracle_abi;
use test_utils::interfaces::pyth_oracle::pyth_oracle_abi;
use test_utils::interfaces::redstone_oracle::{redstone_oracle_abi, RedstoneCore};

use test_utils::setup::common::*;

pub async fn add_asset(symbol: &str) {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let network_name = wallet.provider().unwrap().chain_info().await.unwrap().name;
    let is_testnet = is_testnet(wallet.clone()).await;
    let address: Address = wallet.address().into();
    println!("🔑 Wallet address: {}", address);
    println!("Network name: {}", network_name);
    println!("Is testnet: {}", is_testnet);

    let core_contracts = load_core_contracts(wallet.clone(), is_testnet);

    // Get asset constants based on symbol and network type
    let asset_constants = match (symbol.to_uppercase().as_str(), is_testnet) {
        // Testnet
        ("ETH", true) => &constants::TESTNET_ETH_CONSTANTS,
        // Mainnet
        ("ETH", false) => &constants::MAINNET_ETH_CONSTANTS,
        ("WSTETH", false) => &constants::MAINNET_WSTETH_CONSTANTS,
        ("EZETH", false) => &constants::MAINNET_EZETH_CONSTANTS,
        ("WEETH", false) => &constants::MAINNET_WEETH_CONSTANTS,
        ("RSETH", false) => &constants::MAINNET_RSETH_CONSTANTS,
        ("METH", false) => &constants::MAINNET_METH_CONSTANTS,
        // Add more assets as needed
        (symbol, is_testnet) => panic!(
            "Unsupported asset symbol '{}' for {} network",
            symbol,
            if is_testnet { "testnet" } else { "mainnet" }
        ),
    };

    let existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        symbol: symbol.to_string(),
        asset: match is_testnet {
            true => None,
            false => Some(AssetConfig {
                asset: ContractId::from_str(asset_constants.asset_contract_id.unwrap()).unwrap(),
                asset_id: AssetId::from_str(asset_constants.asset_id.unwrap()).unwrap(),
                fuel_vm_decimals: asset_constants.decimals,
            }),
        },
        pyth_oracle: Some(PythConfig {
            contract: ContractId::from_str(asset_constants.pyth_contract_id).unwrap(),
            price_id: Bits256::from_hex_str(asset_constants.pyth_price_id).unwrap(),
        }),
        redstone_oracle: None,
    };

    // Redstone oracle is not required for initialization
    if existing_asset_to_initialize.asset.is_none()
        || existing_asset_to_initialize.pyth_oracle.is_none()
    {
        // If testnet then cause a failure so it's obvious
        if !is_testnet {
            panic!("Mainnet assets must have an asset and pyth oracle");
        }

        println!("Initializing new asset");
    } else {
        println!("Existing asset to register");
    }

    // Deploy the asset contracts
    let asset_contracts =
        deploy_asset_contracts(&wallet, &existing_asset_to_initialize, false, false).await;

    query_oracles(&asset_contracts, wallet.clone()).await;

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

    write_asset_contracts_to_file(vec![asset_contracts], is_testnet);

    println!("Asset contracts added successfully");
}

fn write_asset_contracts_to_file(
    asset_contracts: Vec<AssetContractsOptionalRedstone<WalletUnlocked>>,
    is_testnet: bool,
) {
    // Read existing contracts.json
    let mut contracts: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(match is_testnet {
            true => TESTNET_CONTRACTS_FILE,
            false => MAINNET_CONTRACTS_FILE,
        })
        .expect("Failed to read contracts.json"),
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
            "symbol": asset_contract.symbol,
            "oracle": asset_contract.oracle.contract_id().to_string(),
            "trove_manager": asset_contract.trove_manager.contract_id().to_string(),
            "asset_contract": asset_contract.asset.contract_id().to_string(),
            "asset_id": format!("0x{}", asset_contract.asset_id.to_string()),
            "pyth_price_id": to_hex_str(&asset_contract.pyth_price_id),
            "pyth_contract": asset_contract.mock_pyth_oracle.contract_id().to_string(),
            "redstone": match &asset_contract.redstone_config {
                Some(redstone_config) => {
                    json!({
                        "redstone_contract": redstone_config.contract.to_string(),
                        "redstone_price_id": redstone_config.price_id.to_string(),
                        "redstone_precision": redstone_config.precision,
                    })
                },
                None => json!(null),
            },
            "fuel_vm_decimals": asset_contract.fuel_vm_decimals,
        }));
    }

    // Update asset_contracts field with the combined list
    contracts["asset_contracts"] = json!(existing_asset_contracts);

    // Write updated contracts back to file
    let mut file = File::create(match is_testnet {
        true => TESTNET_CONTRACTS_FILE,
        false => MAINNET_CONTRACTS_FILE,
    })
    .expect("Failed to open contracts.json for writing");
    file.write_all(serde_json::to_string_pretty(&contracts).unwrap().as_bytes())
        .expect("Failed to write to contracts.json");
}

async fn query_oracles(
    asset_contracts: &AssetContractsOptionalRedstone<WalletUnlocked>,
    wallet: WalletUnlocked,
) {
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
    let mut redstone_contract: Option<RedstoneCore<WalletUnlocked>> = None;
    match &asset_contracts.redstone_config {
        Some(redstone_config) => {
            redstone_contract = Some(RedstoneCore::new(redstone_config.contract, wallet.clone()));
        }
        None => {}
    }

    let current_price = oracle_abi::get_price(
        &asset_contracts.oracle,
        &asset_contracts.mock_pyth_oracle,
        &redstone_contract,
    )
    .await
    .value;

    println!(
        "Current oracle proxy price: {:.9}",
        current_price as f64 / 1_000_000_000.0
    );
    match &asset_contracts.redstone_config {
        Some(redstone_config) => {
            let redstone_precision = redstone_config.precision as usize;
            let current_redstone_price = redstone_oracle_abi::read_prices(
                &redstone_contract.unwrap(),
                vec![redstone_config.price_id],
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
        None => {}
    }
}
pub fn to_hex_str(bits: &Bits256) -> String {
    format!("0x{}", hex::encode(bits.0))
}
