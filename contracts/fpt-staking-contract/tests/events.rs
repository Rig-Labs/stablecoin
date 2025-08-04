use fuels::{prelude::*, types::Identity};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        fpt_staking::fpt_staking_abi,
        token::{token_abi, Token},
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn test_staking_events() {
    let (contracts, admin, mut wallets) = setup_protocol(4, false, true).await;

    // Setup initial conditions
    let mock_token = Token::new(
        contracts.fpt_token.contract.contract_id().clone(),
        wallets.pop().unwrap().clone(),
    );

    let stake_amount = 5 * PRECISION;
    token_abi::mint_to_id(
        &mock_token,
        stake_amount,
        Identity::Address(admin.address().into()),
    )
    .await;

    let mock_token_asset_id = mock_token.contract_id().asset_id(&AssetId::zeroed().into());

    // Test StakeEvent
    let response =
        fpt_staking_abi::stake(&contracts.fpt_staking, mock_token_asset_id, stake_amount)
            .await
            .unwrap();

    let logs = response.decode_logs();
    let stake_event = logs
        .results
        .iter()
        .find(|log| log.as_ref().unwrap().contains("StakeEvent"))
        .expect("StakeEvent not found")
        .as_ref()
        .unwrap();

    assert!(
        stake_event.contains(&admin.address().hash().to_string()),
        "StakeEvent should contain user address"
    );
    assert!(
        stake_event.contains(&stake_amount.to_string()),
        "StakeEvent should contain stake amount"
    );

    // Test UnstakeEvent
    let unstake_amount = 2 * PRECISION;
    let response = fpt_staking_abi::unstake(
        &contracts.fpt_staking,
        &contracts.usdm,
        &mock_token,
        &mock_token,
        unstake_amount,
    )
    .await
    .unwrap();

    let logs = response.decode_logs();
    let unstake_event = logs
        .results
        .iter()
        .find(|log| log.as_ref().unwrap().contains("UnstakeEvent"))
        .expect("UnstakeEvent not found")
        .as_ref()
        .unwrap();

    assert!(
        unstake_event.contains(&admin.address().hash().to_string()),
        "UnstakeEvent should contain user address"
    );
    assert!(
        unstake_event.contains(&unstake_amount.to_string()),
        "UnstakeEvent should contain unstake amount"
    );
}
