use test_utils::{interfaces::trove_manager::trove_manager_abi, setup::common::setup_protocol};

#[tokio::test]
async fn proper_borrow_rates() {
    let (contracts, _admin, mut _wallets) = setup_protocol(10, 5).await;

    let borrow_rate = trove_manager_abi::get_borrowing_rate(&contracts.trove_manager)
        .await
        .value;

    assert_eq!(borrow_rate, 5_000_000);
}
