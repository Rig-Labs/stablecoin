use fuels::{prelude::*, types::{Identity, Bits256}};

use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        fpt_token::{fpt_token_abi, FPTToken},
    },
    setup::common::{deploy_fpt_token},
};

async fn get_contract_instance() -> (
    FPTToken<WalletUnlocked>,
    FPTToken<WalletUnlocked>,
    WalletUnlocked,
) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),                 /* Single wallet */
            Some(1),                 /* Single coin (UTXO) */
            Some(1_000 * PRECISION), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let instance = deploy_fpt_token(&wallet).await;

    let recipient = deploy_fpt_token(&wallet).await;
    
    (instance, recipient, wallet)
}

#[tokio::test]
async fn proper_intialize() {
    let (fpt_token, recipient, _admin) = get_contract_instance().await;
    // println!("0");
    fpt_token_abi::initialize(
        &fpt_token,
        "FPT Token".to_string(),
        "FPT".to_string(),
        recipient.contract_id().into()
    ).await;
    // println!("1");

    let vesting_contract = fpt_token_abi::get_vesting_contract(
        &fpt_token,
    ).await.value;

    // println!("vesting {} {}", vesting_contract, recipient.contract_id().hash());

    assert_eq!(vesting_contract, recipient.contract_id().into());

    let total_supply = fpt_token_abi::total_supply(
        &fpt_token,
    ).await.value;

    // println!("supply {}", total_supply);

    assert_eq!(total_supply, 100_000_000 * 1_000_000_000);
}