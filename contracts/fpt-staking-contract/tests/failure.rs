use fuels::types::{AssetId, Identity};
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
    let (contracts, admin, mut _wallets) = setup_protocol(4, false, true).await;

    let mock_token = Token::new(
        contracts.fpt_token.contract.contract_id().clone(),
        _wallets.pop().unwrap().clone(),
    );
    token_abi::mint_to_id(
        &mock_token,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await;
    let mock_token_asset_id = mock_token.contract_id().asset_id(&AssetId::zeroed().into());

    fpt_staking_abi::stake(&contracts.fpt_staking, mock_token_asset_id, 1 * PRECISION)
        .await
        .unwrap();

    fpt_staking_abi::unstake(
        &contracts.fpt_staking,
        &contracts.usdm,
        &contracts.asset_contracts[0].asset,
        &mock_token,
        1_000 * PRECISION,
    )
    .await
    .expect_err("Unstake incorrect amount allowed");
}
