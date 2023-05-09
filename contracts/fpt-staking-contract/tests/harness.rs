use fuels::{prelude::*, types::Identity};

use test_utils::{
    interfaces::{
        fpt_staking::{fpt_staking_abi, FPTStaking},
        token::{token_abi, Token},
        usdf_token::{usdf_token_abi, USDFToken},
    },
    setup::common::{deploy_fpt_staking, deploy_token, setup_protocol},
};


#[tokio::test]
async fn proper_intialize() {

    let (contracts, admin, mut wallets) = setup_protocol(10, 4, false).await;

    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        5_000_000_000,
        Identity::Address(admin.address().into()),
    )
    .await;

    let pending_rewards_fpt = fpt_staking_abi::get_pending_usdf_gain(&contracts.fpt_staking, Identity::Address(admin.address().into())).await.value;
    assert_eq!(pending_rewards_fpt, 0);

    let pending_rewards_asset = fpt_staking_abi::get_pending_asset_gain(&contracts.fpt_staking, Identity::Address(admin.address().into()), contracts.asset_contracts[0].asset.contract_id().into()).await.value;
    assert_eq!(pending_rewards_asset, 0);

    // how to check token balances of native token 

}