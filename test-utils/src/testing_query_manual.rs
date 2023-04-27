use dotenv::dotenv;
use fuels::{
    prelude::{Address, Bech32Address, Bech32ContractId, Provider, WalletUnlocked},
    types::ContractId,
};

use crate::{
    interfaces::trove_manager::{trove_manager_abi, TroveManagerContract},
    setup::common::ProtocolContracts,
};

const RPC: &str = "beta-3.fuel.network";
// const RPC: &str = "http://localhost:4000";

//#[tokio::test]
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
    let id: Bech32ContractId = "fuel12tcdetv8lgj0mceq5pk75r9d7lcrj7hju76urt90ras3f3yqj8hss2yvxq"
        .parse()
        .expect("Invalid ID");

    let trove_manager = TroveManagerContract::new(id.into(), wallet.clone());

    let res = trove_manager_abi::get_entire_debt_and_coll(
        &trove_manager,
        fuels::types::Identity::Address(wallet.address().into()),
    )
    .await;

    println!("Result: {:#?}", res.value);
}
