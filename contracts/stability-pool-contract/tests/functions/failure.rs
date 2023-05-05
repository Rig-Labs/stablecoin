use crate::utils::setup::setup;
use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, borrow_operations_utils, BorrowOperations},
        oracle::oracle_abi,
        stability_pool::{stability_pool_abi, stability_pool_utils, StabilityPool},
        token::token_abi,
        trove_manager::{trove_manager_abi, trove_manager_utils},
        usdf_token::usdf_token_abi,
    },
    setup::common::{
        add_asset, assert_within_threshold, deploy_token, deploy_usdf_token, setup_protocol,
    },
    utils::with_min_borrow_fee,
};

#[tokio::test]
async fn fails_fake_usdf_deposit() {
    let (contracts, admin, _wallets) = setup_protocol(10, 4, false).await;

    let fake_usdf = deploy_usdf_token(&admin).await;

    usdf_token_abi::initialize(
        &fake_usdf,
        "Fake USDF".to_string(),
        "FUSDF".to_string(),
        ContractId::new([0; 32]),
        Identity::Address(admin.address().into()),
        Identity::Address(admin.address().into()),
    )
    .await;

    usdf_token_abi::mint(
        &fake_usdf,
        5_000 * PRECISION,
        Identity::Address(admin.address().into()),
    )
    .await
    .unwrap();

    stability_pool_abi::provide_to_stability_pool(
        &contracts.stability_pool,
        &fake_usdf,
        &contracts.asset_contracts[0].asset,
        600 * PRECISION,
    )
    .await
    .expect_err("Able to deposit fake USDF into stability pool");
}
