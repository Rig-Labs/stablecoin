use fuels::{prelude::*, types::Identity};
use test_utils::interfaces::borrow_operations::borrow_operations_utils;
use test_utils::interfaces::pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP};
use test_utils::interfaces::redstone_oracle::{redstone_oracle_abi, redstone_price_feed};
use test_utils::{
    data_structures::PRECISION,
    interfaces::hint_helper::hint_helper_abi,
    setup::common::{deploy_hint_helper, setup_protocol},
};

#[ignore = "MemoryWriteOverlap Fuel Error in current version"]
#[tokio::test]
async fn proper_hint_generations() {
    // let (active_pool, _admin) = get_contract_instance().await;
    let (contracts, _admin, mut wallets) = setup_protocol(100, 20, false).await;
    let wallet = wallets.pop().unwrap();

    let hint_helper = deploy_hint_helper(&wallet).await;

    hint_helper_abi::initialize(&hint_helper, contracts.sorted_troves.contract_id().into())
        .await
        .unwrap();

    // create 15 troves each with 600 USDF debt and n * 1000 collateral
    let mut target_address = Identity::Address(wallet.address().into());
    let mut target_address2 = Identity::Address(wallet.address().into());

    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(1),
    )
    .await;

    redstone_oracle_abi::write_prices(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        redstone_price_feed(vec![1]),
    )
    .await;
    redstone_oracle_abi::set_timestamp(
        &contracts.asset_contracts[0].mock_redstone_oracle,
        PYTH_TIMESTAMP,
    )
    .await;

    for i in 1..=15 {
        let wallet = wallets.pop().unwrap();
        let amount = i * 1000 * PRECISION;
        let usdf_amount = 600 * PRECISION;

        if i == 5 {
            target_address = Identity::Address(wallet.address().into());
        }

        if i == 10 {
            target_address2 = Identity::Address(wallet.address().into());
        }

        borrow_operations_utils::mint_token_and_open_trove(
            wallet.clone(),
            &contracts.asset_contracts[0],
            &contracts.borrow_operations,
            &contracts.usdf,
            &contracts.fpt_staking,
            &contracts.active_pool,
            &contracts.sorted_troves,
            amount,
            usdf_amount,
        )
        .await;
    }

    let num_itterations = 25;
    let random_seed = 0;

    let res = hint_helper_abi::get_approx_hint(
        &hint_helper,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].asset_id,
        5000 * PRECISION / 600,
        num_itterations,
        random_seed,
    )
    .await;

    let id = res.value.0;
    assert_eq!(id, target_address);

    let res = hint_helper_abi::get_approx_hint(
        &hint_helper,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].asset_id,
        10000 * PRECISION / 600,
        num_itterations,
        random_seed + 1,
    )
    .await;

    let id = res.value.0;
    assert_eq!(id, target_address2);
}
