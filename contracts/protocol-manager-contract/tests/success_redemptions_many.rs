use fuels::{prelude::AssetId, types::Identity};
use test_utils::data_structures::PRECISION;
use test_utils::interfaces::borrow_operations::borrow_operations_utils;
use test_utils::interfaces::protocol_manager::ProtocolManager;
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi, oracle::oracle_abi, protocol_manager::protocol_manager_abi,
        trove_manager::trove_manager_utils,
    },
    setup::common::setup_protocol,
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn proper_multi_collateral_redemption_from_partially_closed() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 5, true).await;

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 10 * PRECISION).await;
    oracle_abi::set_price(&contracts.aswith_contracts[1].oracle, 10 * PRECISION).await;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet1.clone(),
        &contracts.aswith_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        20_000 * PRECISION,
        10_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet2.clone(),
        &contracts.aswith_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        9_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet3.clone(),
        &contracts.aswith_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        8_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet2.clone(),
        &contracts.aswith_contracts[1],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        15_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet3.clone(),
        &contracts.aswith_contracts[1],
        &contracts.borrow_operations,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        7_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    // 2 Collateral types
    // 1st collateral
    // 20k FUEL > 9k FUEL > 8k FUEL
    // 10k USDF > 5k USDF > 5k USDF + (fees)

    // 2nd collateral
    // 7k stFUEL > 15k stFUEL
    // 5k USDF   > 5k USDF + (fees)

    // Redeeming 10k USDF, so 1,3 and 2,2 should be closed

    oracle_abi::set_price(&contracts.aswith_contracts[0].oracle, 1 * PRECISION).await;
    oracle_abi::set_price(&contracts.aswith_contracts[1].oracle, 1 * PRECISION).await;

    let redemption_amount: u64 = 8_000 * PRECISION;

    let protocol_manager_health1 = ProtocolManager::new(
        contracts.protocol_manager.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    let pre_redemption_active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    protocol_manager_abi::redeem_collateral(
        &protocol_manager_health1,
        redemption_amount,
        20,
        0,
        None,
        None,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.coll_surplus_pool,
        &contracts.default_pool,
        &contracts.active_pool,
        &contracts.sorted_troves,
        &contracts.aswith_contracts,
    )
    .await;

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdf_debt(
        &contracts.active_pool,
        contracts.aswith_contracts[0].asset.contract_id().into(),
    )
    .await
    .value;

    // Total active pool asset should be reduced by the redemption amount
    //  + amount taken from the 2nd collateral type
    assert_eq!(
        active_pool_asset,
        37_000 * PRECISION - redemption_amount + with_min_borrow_fee(5_000 * PRECISION)
    );

    assert_eq!(
        active_pool_debt,
        pre_redemption_active_pool_debt - redemption_amount
            + with_min_borrow_fee(5_000 * PRECISION)
    );

    let provider = healthy_wallet1.provider().unwrap();

    let fuel_asset_id = AssetId::from(*contracts.aswith_contracts[0].asset.contract_id().hash());
    let st_fuel_asset_id = AssetId::from(*contracts.aswith_contracts[1].asset.contract_id().hash());

    let fuel_balance = provider
        .get_asset_balance(healthy_wallet1.address(), fuel_asset_id)
        .await
        .unwrap();

    let st_fuel_balance = provider
        .get_asset_balance(healthy_wallet1.address(), st_fuel_asset_id)
        .await
        .unwrap();

    // TODO Replace with staking contract when implemented
    let staking_balance = provider
        .get_contract_asset_balance(&contracts.fpt_staking.contract_id(), fuel_asset_id)
        .await
        .unwrap();

    let fees2 = provider
        .get_contract_asset_balance(&contracts.fpt_staking.contract_id(), st_fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(
        fuel_balance + st_fuel_balance,
        redemption_amount - staking_balance - fees2
    );

    // Started with 8k portion obsorved by the 2nd collateral type
    trove_manager_utils::assert_trove_coll(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        8_000 * PRECISION + st_fuel_balance + fees2 - redemption_amount,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.aswith_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        with_min_borrow_fee(5_000 * PRECISION) + st_fuel_balance + fees2 - redemption_amount,
    )
    .await;
}
