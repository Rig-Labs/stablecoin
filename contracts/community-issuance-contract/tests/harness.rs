use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        community_issuance::{community_issuance_abi, CommunityIssuance},
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        redstone_oracle::{redstone_oracle_abi, redstone_price_feed},
        stability_pool::{stability_pool_abi, StabilityPool},
        token::token_abi,
    },
    setup::common::setup_protocol,
    utils::print_response,
};

fn abs_dif(a: u64, b: u64) -> u64 {
    if a > b {
        return a - b;
    } else {
        return b - a;
    }
}

#[tokio::test]
async fn test_emissions() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;
    let provider = admin.provider().unwrap();
    let fpt_asset_id = contracts
        .fpt_token
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 0).await;

    let total_emissions = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    println!(
        "FPT balance community issuance STARTING {}",
        total_emissions
    );

    // stability pool depositors, change time, check issuance
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    redstone_oracle_abi::write_prices(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        redstone_price_feed(vec![1]),
    )
    .await;
    redstone_oracle_abi::set_timestamp(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        PYTH_TIMESTAMP,
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 60 * 60 * 24 * 30 * 12)
        .await;

    let res = stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        100 * PRECISION,
    )
    .await
    .unwrap();

    print_response(&res);

    let fpt_balance_community_issuance = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    println!(
        "FPT balance community issuance {}",
        fpt_balance_community_issuance
    );

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions / 4);
    assert!(
        dif < 100_000 * PRECISION,
        "distributed user balance incorrect from 1 year of staking rewards"
    );

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        100 * PRECISION,
    )
    .await
    .unwrap();

    let fpt_balance_user_after_second_claim = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!(
        "user Balance fpt after second deposit (should be same) {}",
        fpt_balance_user_after_second_claim
    );
    assert_eq!(
        fpt_balance_user_after_claim, fpt_balance_user_after_second_claim,
        "double claim staked fpt"
    );
}

#[tokio::test]
async fn test_admin_start_rewards_increase_transition() {
    let (contracts, admin, mut _wallets) = setup_protocol(10, 4, false).await;
    let provider = admin.provider().unwrap();
    let fpt_asset_id = contracts
        .fpt_token
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 0).await;

    let total_emissions = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    // stability pool depositors, change time, check issuance
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    redstone_oracle_abi::write_prices(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        redstone_price_feed(vec![1]),
    )
    .await;
    redstone_oracle_abi::set_timestamp(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        PYTH_TIMESTAMP,
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 31104000 + 1).await;

    community_issuance_abi::start_rewards_increase_transition(
        &contracts.community_issuance,
        604800 + 1,
    )
    .await;

    community_issuance_abi::set_current_time(
        &contracts.community_issuance,
        60 * 60 * 24 * 30 * 12 * 100,
    )
    .await;

    let res = stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        100 * PRECISION,
    )
    .await
    .unwrap();

    print_response(&res);

    let fpt_balance_community_issuance = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    println!(
        "FPT balance community issuance {}",
        fpt_balance_community_issuance
    );

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions);
    assert!(
        dif < 300_000 * PRECISION,
        "distributed user balance incorrect from 100 years of staking rewards with transition"
    );
    //after 100 years with transition almost all rewards should be emitted
}

#[tokio::test]
async fn test_public_start_rewards_increase_transition_after_deadline() {
    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;
    let provider = admin.provider().unwrap();
    let fpt_asset_id = contracts
        .fpt_token
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    let wallet1 = wallets.pop().unwrap();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 0).await;

    let total_emissions = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    // stability pool depositors, change time, check issuance
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    redstone_oracle_abi::write_prices(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        redstone_price_feed(vec![1]),
    )
    .await;
    redstone_oracle_abi::set_timestamp(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        PYTH_TIMESTAMP,
    )
    .await;

    borrow_operations_abi::open_trove(
        &contracts.borrow_operations,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 31104000 + 1).await;

    let community_issuance_wallet1 = CommunityIssuance::new(
        contracts.community_issuance.contract_id().clone(),
        wallet1.clone(),
    );
    // this is to test that anyone can call this function
    community_issuance_abi::public_start_rewards_increase_transition_after_deadline(
        &community_issuance_wallet1,
    )
    .await;

    community_issuance_abi::set_current_time(
        &contracts.community_issuance,
        60 * 60 * 24 * 30 * 12 * 100,
    )
    .await;

    let res = stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        100 * PRECISION,
    )
    .await
    .unwrap();

    print_response(&res);

    let fpt_balance_community_issuance = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    println!(
        "FPT balance community issuance {}",
        fpt_balance_community_issuance
    );

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions);
    assert!(
        dif < 300_000 * PRECISION,
        "distributed user balance incorrect from 100 years of staking rewards with transition"
    );
    //after 100 years with transition almost all rewards should be emitted
}

#[tokio::test]
async fn test_emissions_multiple_deposits() {
    let (contracts, admin, mut wallets) = setup_protocol(100, 4, false).await;

    let provider = admin.provider().unwrap();
    let fpt_asset_id = contracts
        .fpt_token
        .contract_id()
        .asset_id(&AssetId::zeroed().into())
        .into();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 0).await;

    let total_emissions = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();
    let wallet3 = wallets.pop().unwrap();

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(wallet1.address().into()),
    )
    .await;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(wallet2.address().into()),
    )
    .await;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(wallet3.address().into()),
    )
    .await;

    let borrow_operations_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet1.clone(),
    );
    let borrow_operations_wallet2 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet2.clone(),
    );
    let borrow_operations_wallet3 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet3.clone(),
    );

    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    redstone_oracle_abi::write_prices(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        redstone_price_feed(vec![1]),
    )
    .await;
    redstone_oracle_abi::set_timestamp(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        PYTH_TIMESTAMP,
    )
    .await;

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet2,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet3,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_200 * PRECISION,
        600 * PRECISION,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await
    .unwrap();

    let stability_pool_wallet1 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        wallet1.clone(),
    );
    let stability_pool_wallet2 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        wallet2.clone(),
    );
    let stability_pool_wallet3 = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        wallet3.clone(),
    );
    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet1,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();
    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();
    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet3,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    community_issuance_abi::set_current_time(&contracts.community_issuance, 60 * 60 * 24 * 30 * 12)
        .await;

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet1,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();
    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet2,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &stability_pool_wallet3,
        &contracts.community_issuance,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        300 * PRECISION,
    )
    .await
    .unwrap();

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(wallet1.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user1 Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions / 4 / 3);
    assert!(
        dif < 100_000 * PRECISION,
        "distributed user balance incorrect from 1 year of staking rewards"
    );

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(wallet2.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user2 Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions / 4 / 3);
    assert!(
        dif < 100_000 * PRECISION,
        "distributed user balance incorrect from 1 year of staking rewards"
    );

    let fpt_balance_user_after_claim = provider
        .get_asset_balance(wallet3.address().into(), fpt_asset_id)
        .await
        .unwrap();

    println!("user3 Balance fpt {}", fpt_balance_user_after_claim);

    let dif = abs_dif(fpt_balance_user_after_claim, total_emissions / 4 / 3);
    assert!(
        dif < 100_000 * PRECISION,
        "distributed user balance incorrect from 1 year of staking rewards"
    );
}
