use dotenv::dotenv;
use fuels::prelude::{Address, Provider, WalletUnlocked};

use crate::setup::common::deploy_and_initialize_all;

// const RPC: &str = "beta-3.fuel.network";
const RPC: &str = "http://localhost:4000";

// #[tokio::test]
pub async fn deploy() {
    //--------------- WALLET ---------------
    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let secret = match std::env::var("SECRET") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };

    let wallet = WalletUnlocked::new_from_mnemonic_phrase_with_path(
        &secret,
        Some(provider.clone()),
        "m/44'/1179993420'/0'/0/0",
    )
    .unwrap();

    let address = Address::from(wallet.address());
    println!("🔑 Wallet address: {}", address);

    let contracts = deploy_and_initialize_all(wallet, 100, true, false).await;

    println!(
        "Borrow operations: {}",
        contracts.borrow_operations.contract_id()
    );
    println!(
        "Oracle: {}",
        contracts.asset_contracts[0].oracle.contract_id()
    );
    println!(
        "Sorted Troves: {}",
        contracts.asset_contracts[0].sorted_troves.contract_id()
    );
    println!(
        "Trove Manager: {}",
        contracts.asset_contracts[0].trove_manager.contract_id()
    );
    println!("Fuel: {}", contracts.asset_contracts[0].asset.contract_id());
    println!("Usdf: {}", contracts.usdf.contract_id());
    println!(
        "Active Pool: {}",
        contracts.asset_contracts[0].active_pool.contract_id()
    );
    println!("Stability Pool: {}", contracts.stability_pool.contract_id());
    println!(
        "Default Pool: {}",
        contracts.asset_contracts[0].default_pool.contract_id()
    );
    println!(
        "Collateral Surplus Pool: {}",
        contracts.asset_contracts[0].coll_surplus_pool.contract_id()
    );
}
