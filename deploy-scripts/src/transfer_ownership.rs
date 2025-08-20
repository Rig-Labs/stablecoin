use crate::utils::utils::{is_testnet, load_core_contracts, setup_wallet};
use dotenv::dotenv;
use fuels::{accounts::ViewOnlyAccount, types::Identity};
use test_utils::interfaces::{
    borrow_operations::borrow_operations_abi, protocol_manager::protocol_manager_abi,
};

pub async fn transfer_owner(new_owner: &str) {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let address = wallet.address();
    println!("🔑 Wallet address: {}", address);

    let is_testnet = is_testnet(wallet.clone()).await;
    let core_contracts = load_core_contracts(wallet.clone(), is_testnet);

    println!(
        "Are you sure you want to transfer ownership to {}? (y/n)",
        new_owner
    );
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return;
    }

    // Convert string address to Identity
    let new_owner_identity = Identity::Address(new_owner.parse().expect("Invalid address format"));

    let _ = borrow_operations_abi::transfer_owner(
        &core_contracts.borrow_operations,
        new_owner_identity,
    )
    .await
    .unwrap();

    let _ =
        protocol_manager_abi::transfer_owner(&core_contracts.protocol_manager, new_owner_identity)
            .await
            .unwrap();

    println!("Ownership transferred successfully to {}", new_owner);
}
