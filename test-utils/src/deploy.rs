use dotenv::dotenv;
use fuels::prelude::{Address, Provider, WalletUnlocked};
use pbr::ProgressBar;

use crate::setup::common::{
    deploy_active_pool, deploy_borrow_operations, deploy_coll_surplus_pool, deploy_default_pool,
    deploy_oracle, deploy_sorted_troves, deploy_token, deploy_trove_manager_contract,
    deploy_usdf_token,
};

const RPC: &str = "beta-3.fuel.network";

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

    let mut pb = ProgressBar::new(10);
    let borrow_operations = deploy_borrow_operations(&wallet).await;
    pb.inc();
    let oracle_instance = deploy_oracle(&wallet).await;
    pb.inc();
    let sorted_troves = deploy_sorted_troves(&wallet).await;
    pb.inc();
    let trove_manger = deploy_trove_manager_contract(&wallet).await;
    pb.inc();
    let fuel = deploy_token(&wallet).await;
    pb.inc();
    let usdf = deploy_usdf_token(&wallet).await;
    pb.inc();
    let active_pool = deploy_active_pool(&wallet).await;
    pb.inc();
    let stability_pool = deploy_active_pool(&wallet).await;
    pb.inc();
    let default_pool = deploy_default_pool(&wallet).await;
    pb.inc();
    let coll_surplus_pool = deploy_coll_surplus_pool(&wallet).await;
    pb.finish();

    println!("Borrow operations: {}", borrow_operations.contract_id());
    println!("Oracle: {}", oracle_instance.contract_id());
    println!("Sorted Troves: {}", sorted_troves.contract_id());
    println!("Trove Manager: {}", trove_manger.contract_id());
    println!("Fuel: {}", fuel.contract_id());
    println!("Usdf: {}", usdf.contract_id());
    println!("Active Pool: {}", active_pool.contract_id());
    println!("Stability Pool: {}", stability_pool.contract_id());
    println!("Default Pool: {}", default_pool.contract_id());
    println!(
        "Collateral Surplus Pool: {}",
        coll_surplus_pool.contract_id()
    );
}
