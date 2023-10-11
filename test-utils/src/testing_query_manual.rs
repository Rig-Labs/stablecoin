use crate::interfaces::oracle::{oracle_abi, Oracle};
// use crate::interfaces::trove_manager::{trove_manager_abi, TroveManagerContract};
use dotenv::dotenv;
use fuels::prelude::{Bech32ContractId, Provider, WalletUnlocked};

const RPC: &str = "beta-4.fuel.network";
// const RPC: &str = "http://localhost:4000";

// #[tokio::test]
pub async fn testing_query() {
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

    println!("Wallet address: {}", wallet.address());
    let id: Bech32ContractId = "fuel1xaep9urp7ududltl74ej25lw9gvqd0qfsy3taljwx6ts089exevqskucmh"
        .parse()
        .expect("Invalid ID");

    let oracle = Oracle::new(id, wallet.clone());

    let res = oracle_abi::get_price(&oracle).await;

    println!("Result: {:#?}", res.value);
}
