use crate::utils::utils::*;
use dotenv::dotenv;
use fuels::prelude::*;
use test_utils::interfaces::borrow_operations::borrow_operations_abi;
use test_utils::interfaces::stability_pool::stability_pool_abi;

use test_utils::data_structures::PRECISION;

pub async fn sanity_check() {
    dotenv().ok();

    let wallet = setup_wallet().await;
    let address = wallet.address();
    println!("🔑 Wallet address: {}", address);

    let core_contracts = load_core_contracts(wallet.clone());

    let provider = wallet.provider().unwrap();

    let community_issuance_balance = provider
        .get_contract_asset_balance(
            core_contracts.community_issuance.contract_id(),
            core_contracts.fpt_asset_id.into(),
        )
        .await
        .unwrap();

    println!(
        "Community issuance fpt balance: {}",
        community_issuance_balance
    );

    let vesting_contract_balance = provider
        .get_contract_asset_balance(
            core_contracts.vesting_contract.contract_id(),
            core_contracts.fpt_asset_id.into(),
        )
        .await
        .unwrap();

    println!("Vesting contract fpt balance: {}", vesting_contract_balance);

    println!("Are you sure you want to run the sanity check? (y/n)");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return;
    }

    let collateral_amount = 4000 * PRECISION;
    let debt = 1000 * PRECISION;

    let balance = provider
        .get_asset_balance(
            wallet.address().into(),
            core_contracts.asset_contracts[0].asset_id.into(),
        )
        .await
        .unwrap();
    assert!(balance >= collateral_amount);
    println!("Balance: {}", balance);

    let res = borrow_operations_abi::open_trove(
        &core_contracts.borrow_operations,
        &core_contracts.asset_contracts[0].oracle,
        &core_contracts.asset_contracts[0].mock_pyth_oracle,
        &core_contracts.asset_contracts[0].mock_redstone_oracle,
        &core_contracts.asset_contracts[0].asset,
        &core_contracts.usdf,
        &core_contracts.fpt_staking,
        &core_contracts.sorted_troves,
        &core_contracts.asset_contracts[0].trove_manager,
        &core_contracts.active_pool,
        collateral_amount,
        debt,
        fuels::types::Identity::Address(Address::zeroed()),
        fuels::types::Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    println!("Open trove res: {:?}", res.decode_logs());

    let usdf_balance = provider
        .get_asset_balance(wallet.address().into(), core_contracts.usdf_asset_id.into())
        .await
        .unwrap();

    println!("USDF balance: {}", usdf_balance);

    stability_pool_abi::provide_to_stability_pool(
        &core_contracts.stability_pool,
        &core_contracts.community_issuance,
        &core_contracts.usdf,
        &core_contracts.asset_contracts[0].asset,
        usdf_balance,
    )
    .await
    .unwrap();

    let stability_pool_balance = provider
        .get_asset_balance(
            wallet.address().into(),
            core_contracts.asset_contracts[0].asset_id.into(),
        )
        .await
        .unwrap();

    println!("Stability pool balance: {}", stability_pool_balance);
}
