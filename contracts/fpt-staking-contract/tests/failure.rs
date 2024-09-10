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
async fn fails_unstake_wrong_amount() {
    let (contracts, admin, mut _wallets) = setup_protocol(10, 4, false, true).await;

    let mock_token = Token::new(
        contracts.fpt_token.contract_id().clone(),
        _wallets.pop().unwrap().clone(),
    );
    token_abi::mint_to_id(
        &mock_token,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;

    fpt_staking_abi::stake(&contracts.fpt_staking, &mock_token, 1 * PRECISION).await;

    fpt_staking_abi::unstake(
        &contracts.fpt_staking,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        &mock_token,
        1_000 * PRECISION,
    )
    .await
    .expect_err("Unstake incorrect amount allowed");
}
