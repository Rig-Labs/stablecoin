use fuels::prelude::*;

use test_utils::{
    data_structures::PRECISION, interfaces::fpt_token::fpt_token_abi, setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_intialize() {
    let (contracts, admin, _wallets) = setup_protocol(4, false, false).await;
    let provider = admin.provider().unwrap();
    let fpt_asset_id = contracts.fpt_asset_id;

    let vesting_contract = fpt_token_abi::get_vesting_contract(&contracts.fpt_token)
        .await
        .value;

    assert_eq!(
        vesting_contract,
        contracts.vesting_contract.contract.contract_id().into()
    );

    let fpt_balance_vesting = provider
        .get_contract_asset_balance(
            contracts.vesting_contract.contract.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    assert_eq!(
        fpt_balance_vesting,
        0,
        "invalid vesting balance initialized"
    );

    let fpt_balance_community_issuance = provider
        .get_contract_asset_balance(
            contracts.community_issuance.contract.contract_id().into(),
            fpt_asset_id,
        )
        .await
        .unwrap();

    assert_eq!(
        fpt_balance_community_issuance,
        0,
        "invalid community issuance balance initialized"
    );

    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fpt_asset_id)
        .await
        .unwrap();

    assert_eq!(
        admin_balance,
        1 * PRECISION,
        "invalid admin balance after initialized"
    );

    let total_supply = fpt_token_abi::total_supply(&contracts.fpt_token)
        .await
        .value;

    assert_eq!(total_supply, Some(1 * PRECISION));
}
