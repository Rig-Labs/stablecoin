use fuels::types::Identity;
use test_utils::data_structures::{ContractInstance, PRECISION};
use test_utils::interfaces::borrow_operations::borrow_operations_utils;
use test_utils::interfaces::oracle::oracle_abi;
use test_utils::interfaces::protocol_manager::ProtocolManager;
use test_utils::interfaces::pyth_oracle::PYTH_TIMESTAMP;
use test_utils::{
    interfaces::{
        active_pool::active_pool_abi,
        protocol_manager::protocol_manager_abi,
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed},
        trove_manager::trove_manager_utils,
    },
    setup::common::setup_protocol,
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn proper_multi_collateral_redemption_from_partially_closed() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, true, false).await;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[1].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[1].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet1.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        20_000 * PRECISION,
        10_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet2.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        9_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet3.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        8_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet2.clone(),
        &contracts.asset_contracts[1],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        15_000 * PRECISION,
        5_000 * PRECISION,
    )
    .await;

    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet3.clone(),
        &contracts.asset_contracts[1],
        &contracts.borrow_operations,
        &contracts.usdm,
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
    // 10k USDM > 5k USDM > 5k USDM + (fees)

    // 2nd collateral
    // 7k mock2 > 15k mock2
    // 5k USDM   > 5k USDM + (fees)

    // Redeeming 10k USDM, so 1,3 and 2,2 should be closed

    let redemption_amount: u64 = 8_000 * PRECISION;

    let protocol_manager_health1 = ContractInstance::new(
        ProtocolManager::new(
            contracts.protocol_manager.contract.contract_id().clone(),
            healthy_wallet1.clone(),
        ),
        contracts.protocol_manager.implementation_id,
    );

    let pre_redemption_active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
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
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.coll_surplus_pool,
        &contracts.default_pool,
        &contracts.active_pool,
        &contracts.sorted_troves,
        &contracts.asset_contracts,
    )
    .await;

    let active_pool_asset = active_pool_abi::get_asset(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;

    let active_pool_debt = active_pool_abi::get_usdm_debt(
        &contracts.active_pool,
        contracts.asset_contracts[0].asset_id.into(),
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

    let mock_asset_id = contracts.asset_contracts[0].asset_id;
    let st_mock_asset_id = contracts.asset_contracts[1].asset_id;

    let mock_balance = provider
        .get_asset_balance(healthy_wallet1.address(), mock_asset_id)
        .await
        .unwrap();

    let st_mock_balance = provider
        .get_asset_balance(healthy_wallet1.address(), st_mock_asset_id)
        .await
        .unwrap();

    let staking_balance = provider
        .get_contract_asset_balance(&contracts.fpt_staking.contract.contract_id(), mock_asset_id)
        .await
        .unwrap();

    let fees2 = provider
        .get_contract_asset_balance(
            &contracts.fpt_staking.contract.contract_id(),
            st_mock_asset_id,
        )
        .await
        .unwrap();

    assert_eq!(
        mock_balance + st_mock_balance,
        redemption_amount - staking_balance - fees2
    );

    // Started with 8k portion obsorved by the 2nd collateral type
    trove_manager_utils::assert_trove_coll(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        8_000 * PRECISION + st_mock_balance + fees2 - redemption_amount,
    )
    .await;

    trove_manager_utils::assert_trove_debt(
        &contracts.asset_contracts[0].trove_manager,
        Identity::Address(healthy_wallet3.address().into()),
        with_min_borrow_fee(5_000 * PRECISION) + st_mock_balance + fees2 - redemption_amount,
    )
    .await;
}
