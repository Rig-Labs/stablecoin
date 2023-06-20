use fuels::{prelude::*, types::{Identity, Bits256}};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        fpt_token::{fpt_token_abi, FPTToken},
    },
    setup::common::{setup_protocol},
};

#[tokio::test]
async fn proper_intialize() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;
    let provider = admin.provider().unwrap();
    let fpt_asset_id = AssetId::from(*contracts.fpt_token.contract_id().hash());


    let vesting_contract = fpt_token_abi::get_vesting_contract(
        &contracts.fpt_token,
    ).await.value;

    // println!("vesting {} {}", vesting_contract, recipient.contract_id().hash());

    assert_eq!(vesting_contract, contracts.usdf.contract_id().into());

    let fpt_balance_vesting = provider
    .get_contract_asset_balance(contracts.usdf.contract_id().into(), fpt_asset_id)
    .await
    .unwrap();

    assert_eq!(fpt_balance_vesting, 68_000_000_000_000_000, "invalid vesting balance initialized");

    let fpt_balance_community_issuance = provider
    .get_contract_asset_balance(contracts.community_issuance.contract_id().into(), fpt_asset_id)
    .await
    .unwrap();

    assert_eq!(fpt_balance_community_issuance, 32_000_000_000_000_000, "invalid community issuance balance initialized");

    let total_supply = fpt_token_abi::total_supply(
        &contracts.fpt_token,
    ).await.value;

    // println!("supply {}", total_supply);

    assert_eq!(total_supply, 100_000_000 * 1_000_000_000);
}