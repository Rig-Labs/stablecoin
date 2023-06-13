use fuels::{prelude::*, types::Identity};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        stability_pool::{stability_pool_abi, StabilityPool},
        usdf_token::usdf_token_abi,
    },
    setup::common::{deploy_usdf_token, setup_protocol},
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

#[tokio::test]
async fn fails_unauthorized() {
    let (contracts, _admin, mut wallets) = setup_protocol(10, 4, false).await;

    let attacker = wallets.pop().unwrap();

    let stability_pool_attacker = StabilityPool::new(
        contracts.stability_pool.contract_id().clone(),
        attacker.clone(),
    );

    stability_pool_abi::initialize(
        &stability_pool_attacker,
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
    )
    .await
    .expect_err("Able to initialize stability pool with unauthorized address");

    stability_pool_abi::add_asset(
        &stability_pool_attacker,
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
        ContractId::new([0; 32].into()),
    )
    .await
    .expect_err("Able to add asset with unauthorized address");
}
