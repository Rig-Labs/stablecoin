use fuels::types::{Address, Identity};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        borrow_operations::{borrow_operations_abi, BorrowOperations},
        oracle::oracle_abi,
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed, PYTH_TIMESTAMP},
        token::token_abi,
        trove_manager::trove_manager_abi,
    },
    setup::common::setup_protocol,
};

#[tokio::test]
async fn fails_to_liquidate_trove_not_under_mcr() {
    let (contracts, _admin, mut wallets) = setup_protocol(5, false, false).await;

    oracle_abi::set_debug_timestamp(&contracts.asset_contracts[0].oracle, PYTH_TIMESTAMP).await;
    pyth_oracle_abi::update_price_feeds(
        &contracts.asset_contracts[0].mock_pyth_oracle,
        pyth_price_feed(10),
    )
    .await;

    let wallet1 = wallets.pop().unwrap();

    let balance = 25_000 * PRECISION;
    token_abi::mint_to_id(
        &contracts.asset_contracts[0].asset,
        balance,
        Identity::Address(wallet1.address().into()),
    )
    .await;

    let borrow_operations_wallet1 = BorrowOperations::new(
        contracts.borrow_operations.contract_id().clone(),
        wallet1.clone(),
    );

    borrow_operations_abi::open_trove(
        &borrow_operations_wallet1,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.asset_contracts[0].asset,
        &contracts.usdf,
        &contracts.fpt_staking,
        &contracts.sorted_troves,
        &contracts.asset_contracts[0].trove_manager,
        &contracts.active_pool,
        1_100 * PRECISION,
        1_000 * PRECISION,
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .unwrap();

    trove_manager_abi::liquidate(
        &contracts.asset_contracts[0].trove_manager,
        &contracts.community_issuance,
        &contracts.stability_pool,
        &contracts.asset_contracts[0].oracle,
        &contracts.asset_contracts[0].mock_pyth_oracle,
        &contracts.asset_contracts[0].mock_redstone_oracle,
        &contracts.sorted_troves,
        &contracts.active_pool,
        &contracts.default_pool,
        &contracts.coll_surplus_pool,
        &contracts.usdf,
        Identity::Address(wallet1.address().into()),
        Identity::Address(Address::zeroed()),
        Identity::Address(Address::zeroed()),
    )
    .await
    .expect_err("Improper liquidation of trove not below MCR");
}
