use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::{ContractInstance, PRECISION},
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        fpt_staking::{fpt_staking_abi, FPTStaking},
        oracle::oracle_abi,
        protocol_manager::{protocol_manager_abi, ProtocolManager},
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        token::{token_abi, Token},
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_intialize() {
    let (contracts, admin, _wallets) = setup_protocol(4, false, true).await;
    println!("admin address {:?}", admin.address());
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let pending_rewards_fpt = fpt_staking_abi::get_pending_usdm_gain(
        &contracts.fpt_staking,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;
    assert_eq!(pending_rewards_fpt, 0);

    let pending_rewards_asset = fpt_staking_abi::get_pending_asset_gain(
        &contracts.fpt_staking,
        Identity::Address(admin.address().into()),
        contracts.asset_contracts[0].asset_id.into(),
    )
    .await
    .value;
    assert_eq!(pending_rewards_asset, 0);
}

#[tokio::test]
async fn proper_staking_deposit() {
    let (contracts, admin, mut _wallets) = setup_protocol(4, false, true).await;

    let provider = admin.provider().unwrap();

    let fpt_asset_id = contracts.fpt_asset_id;

    let mock_token = Token::new(
        contracts.fpt_token.contract.contract_id().clone(),
        _wallets.pop().unwrap().clone(),
    );
    token_abi::mint_to_id(
        &mock_token,
        5 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    let mock_token_asset_id = mock_token.contract_id().asset_id(&AssetId::zeroed().into());

    fpt_staking_abi::stake(&contracts.fpt_staking, mock_token_asset_id, 1 * PRECISION)
        .await
        .unwrap();

    let fpt_balance = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(fpt_balance, 4 * PRECISION, "FPT Balance is wrong");
}

#[tokio::test]
async fn proper_staking_multiple_positions() {
    let (contracts, admin, mut wallets) = setup_protocol(4, false, true).await;

    let provider = admin.provider().unwrap();

    let fpt_asset_id = contracts.fpt_asset_id;

    let usdm_asset_id = contracts.usdm_asset_id;

    let healthy_wallet1 = wallets.pop().unwrap();
    let healthy_wallet2 = wallets.pop().unwrap();
    let healthy_wallet3 = wallets.pop().unwrap();

    let mock_token = Token::new(
        contracts.fpt_token.contract.contract_id().clone(),
        healthy_wallet1.clone(),
    );

    token_abi::mint_to_id(
        &mock_token,
        5 * PRECISION,
        Identity::Address(healthy_wallet1.address().into()),
    )
    .await;

    token_abi::mint_to_id(
        &mock_token,
        5 * PRECISION,
        Identity::Address(healthy_wallet2.address().into()),
    )
    .await;

    let fpt_staking_healthy_wallet1 = ContractInstance::new(
        FPTStaking::new(
            contracts.fpt_staking.contract.contract_id().clone(),
            healthy_wallet1.clone(),
        ),
        contracts.fpt_staking.implementation_id,
    );

    let fpt_staking_healthy_wallet2 = ContractInstance::new(
        FPTStaking::new(
            contracts.fpt_staking.contract.contract_id().clone(),
            healthy_wallet2.clone(),
        ),
        contracts.fpt_staking.implementation_id,
    );

    let mock_token_asset_id = mock_token.contract_id().asset_id(&AssetId::zeroed().into());

    fpt_staking_abi::stake(
        &fpt_staking_healthy_wallet1,
        mock_token_asset_id,
        1 * PRECISION,
    )
    .await
    .unwrap();

    fpt_staking_abi::stake(
        &fpt_staking_healthy_wallet2,
        mock_token_asset_id,
        1 * PRECISION,
    )
    .await
    .unwrap();

    let fpt_balance_user1 = provider
        .get_asset_balance(healthy_wallet1.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(fpt_balance_user1, 4 * PRECISION, "FPT Balance is wrong");

    let fpt_balance_user1 = provider
        .get_asset_balance(healthy_wallet2.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(fpt_balance_user1, 4 * PRECISION, "FPT Balance is wrong");

    // basically we are going to open a trove, and through that generate some revenue for staking

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        40_000 * PRECISION,
        Identity::Address(healthy_wallet3.address().into()),
    )
    .await;

    // let asset_user_balance = provider
    // .get_asset_balance(healthy_wallet3.address().into(), asset_id)
    // .await
    // .unwrap();

    // println!("Asset balance user {}", asset_user_balance);

    let borrow_operations_healthy_wallet3 = ContractInstance::new(
        BorrowOperations::new(
            contracts.borrow_operations.contract.contract_id().clone(),
            healthy_wallet3.clone(),
        ),
        contracts.borrow_operations.implementation_id,
    );

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    let _open_trove = borrow_operations_abi::open_trove(
        &borrow_operations_healthy_wallet3,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        40_000 * PRECISION,
        20_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    let usdm_in_staking_balance = provider
        .get_contract_asset_balance(&contracts.fpt_staking.contract.contract_id(), usdm_asset_id)
        .await
        .unwrap();

    // println!("USDM balance in staking contract {}", usdm_in_staking_balance);

    // let initial_usdm_user_balance = provider
    // .get_asset_balance(healthy_wallet3.address().into(), usdm_asset_id)
    // .await
    // .unwrap();

    // println!("USDM balance user {}", initial_usdm_user_balance);

    // let asset_user_balance = provider
    // .get_asset_balance(healthy_wallet3.address().into(), asset_id)
    // .await
    // .unwrap();

    // println!("Asset balance user {}", asset_user_balance);

    let redeem_amount = 10_000 * PRECISION;

    let protocol_manager_healthy_wallet3 = ContractInstance::new(
        ProtocolManager::new(
            contracts.protocol_manager.contract.contract_id().clone(),
            healthy_wallet3.clone(),
        ),
        contracts.protocol_manager.implementation_id,
    );
    protocol_manager_abi::redeem_collateral(
        &protocol_manager_healthy_wallet3,
        redeem_amount,
        10,
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

    let asset_in_staking_balance = provider
        .get_contract_asset_balance(
            &contracts.fpt_staking.contract.contract_id(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    // println!("ASSET balance in staking contract {}", asset_in_staking_balance);

    let _res1 = fpt_staking_abi::unstake(
        &fpt_staking_healthy_wallet1,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        &mock_token,
        500_000_000,
    )
    .await;

    let _res2 = fpt_staking_abi::unstake(
        &fpt_staking_healthy_wallet2,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        &mock_token,
        500_000_000,
    )
    .await;

    // println!("unstake");
    // print_response(&res);
    // println!("{:?}", &res.receipts);

    let fpt_balance_user1 = provider
        .get_asset_balance(healthy_wallet1.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(fpt_balance_user1, 4_500_000_000, "FPT Balance is wrong");

    let fpt_balance_user2 = provider
        .get_asset_balance(healthy_wallet2.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(fpt_balance_user2, 4_500_000_000, "FPT Balance is wrong");

    let usdm_user1_balance = provider
        .get_asset_balance(healthy_wallet1.address().into(), usdm_asset_id)
        .await
        .unwrap();

    // println!("USDM balance user {}", usdm_user1_balance);

    let usdm_user2_balance = provider
        .get_asset_balance(healthy_wallet2.address().into(), usdm_asset_id)
        .await
        .unwrap();

    // println!("USDM balance user {}", usdm_user2_balance);

    assert_eq!(
        usdm_user1_balance, usdm_user2_balance,
        "users usdm rewards don't match"
    );

    // println!("Should receive (together) usdm {}", usdm_in_staking_balance);

    assert_eq!(
        usdm_user1_balance + usdm_user2_balance,
        usdm_in_staking_balance,
        "Users did not receive exactly all the usdm staking rewards"
    );

    let asset_user1_balance = provider
        .get_asset_balance(
            healthy_wallet1.address().into(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    // println!("Asset balance user {}", asset_user1_balance);

    let asset_user2_balance = provider
        .get_asset_balance(
            healthy_wallet2.address().into(),
            contracts.asset_contracts[0].asset_id,
        )
        .await
        .unwrap();

    // println!("Asset balance user {}", asset_user2_balance);

    assert_eq!(
        asset_user1_balance, asset_user2_balance,
        "users asset rewards balance don't match"
    );

    // println!("Should receive (together) asset {}", asset_in_staking_balance);

    // let asset_in_staking_balance_after = provider
    // .get_contract_asset_balance(&contracts.fpt_staking.contract_id(), asset_id)
    // .await
    // .unwrap();

    // println!("ASSET balance in staking contract {}", asset_in_staking_balance_after);

    assert_eq!(
        asset_user1_balance + asset_user2_balance,
        asset_in_staking_balance,
        "Users did not receive exactly all the asset staking rewards"
    );
}
