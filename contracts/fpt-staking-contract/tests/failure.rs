use fuels::{prelude::*, types::Identity};
use test_utils::{
    interfaces::{
        fpt_staking::{fpt_staking_abi},
        token::{token_abi},
    },
    setup::common::{setup_protocol},
};

#[tokio::test]
async fn fails_unstake_wrong_amount(){

    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;

    token_abi::mint_to_id(
        &contracts.fpt,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    fpt_staking_abi::stake(
        &contracts.fpt_staking,
        &contracts.fpt,
        1_000_000_000,
    ).await;

   fpt_staking_abi::unstake(
        &contracts.fpt_staking,
        &contracts.usdf,
        &contracts.asset_contracts[0].asset,
        &contracts.fpt,
        1_000_000_000_000,
    ).await.expect_err("Unstake incorrect amount allowed");

}