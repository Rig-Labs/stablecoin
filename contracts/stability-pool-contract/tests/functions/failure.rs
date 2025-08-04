use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::{ContractInstance, PRECISION},
    interfaces::{
        borrow_operations::borrow_operations_utils,
        oracle::oracle_abi,
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed, pyth_price_no_precision_with_time, PYTH_TIMESTAMP,
        },
        stability_pool::{stability_pool_abi, StabilityPool},
        usdm_token::usdm_token_abi,
    },
    setup::common::{deploy_usdm_token, setup_protocol},
};

#[tokio::test]
async fn fails_fake_usdm_deposit() {
    let (contracts, admin, _wallets) = setup_protocol(4, false, false).await;

    let fake_usdm = deploy_usdm_token(&admin).await;

    usdm_token_abi::initialize(
        &fake_usdm,
        ContractId::zeroed(),
        Identity::Address(admin.address().into()),
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap();

    usdm_token_abi::mint(
        &fake_usdm,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &fake_usdm,
        &contracts.asset_contracts[0].asset,
        600 * PRECISION,
    )
    .await
    .expect_err("Able to deposit fake USDM into stability pool");
}

#[tokio::test]
async fn fails_unauthorized() {
    let (contracts, _admin, mut wallets) = setup_protocol(4, false, false).await;

    let attacker = wallets.pop().unwrap();

    let stability_pool_attacker = ContractInstance::new(
        StabilityPool::new(
            contracts.stability_pool.contract.contract_id().clone(),
            attacker.clone(),
        ),
        contracts.stability_pool.implementation_id,
    );

    stability_pool_abi::initialize(
        &stability_pool_attacker,
        ContractId::zeroed(),
        ContractId::zeroed(),
        ContractId::zeroed(),
        ContractId::zeroed(),
        ContractId::zeroed(),
    )
    .await
    .expect_err("Able to initialize stability pool with unauthorized address");

    stability_pool_abi::add_asset(
        &stability_pool_attacker,
        ContractId::zeroed(),
        AssetId::zeroed(),
        ContractId::zeroed(),
    )
    .await
    .expect_err("Able to add asset with unauthorized address");
}

#[tokio::test]
async fn fails_withdraw_with_undercollateralized_trove() {
    let (contracts, admin, mut wallets) = setup_protocol(4, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

    let liquidated_wallet = wallets.pop().unwrap();
    let healthy_wallet = wallets.pop().unwrap();
    // Open one trove that is not undercollateralized
    borrow_operations_utils::mint_token_and_open_trove(
        healthy_wallet.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        100_000 * PRECISION,
        3_000 * PRECISION,
    )
    .await;

    // Admin opens a trove and deposits to stability pool
    borrow_operations_utils::mint_token_and_open_trove(
        admin.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        6_000 * PRECISION,
        3_000 * PRECISION,
    )
    .await;

    let init_stability_deposit = 2_000 * PRECISION;
    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        init_stability_deposit,
    )
    .await
    .unwrap();

    // Liquidated wallet opens a trove with low collateral ratio
    borrow_operations_utils::mint_token_and_open_trove(
        liquidated_wallet.clone(),
        &contracts.asset_contracts[0],
        &contracts.borrow_operations,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.active_pool,
        &contracts.sorted_troves,
        1_100 * PRECISION,
        1_000 * PRECISION,
    )
    .await;

    // Update price to make the liquidated_wallet's trove undercollateralized
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 1).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_no_precision_with_time(PRECISION / 2, PYTH_TIMESTAMP + 1), // Price drops by half
    )
    .await;

    // Try to withdraw from stability pool
    let withdraw_result = stability_pool_abi::withdraw_from_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].trove_manager,
        1_000 * PRECISION,
    )
    .await;

    // Assert that the withdrawal fails
    assert!(
        withdraw_result.is_err(),
        "Withdrawal should fail when there's an undercollateralized trove"
    );

    // price returns to normal
    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP + 2).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

    // Try to withdraw again
    let withdraw_result = stability_pool_abi::withdraw_from_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].trove_manager,
        1_000 * PRECISION,
    )
    .await;

    // Assert that the withdrawal succeeds
    assert!(
        withdraw_result.is_ok(),
        "Withdrawal should succeed when there's no undercollateralized trove"
    );
}
