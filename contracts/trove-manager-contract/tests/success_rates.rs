use test_utils::{interfaces::trove_manager::trove_manager_abi, setup::common::setup_protocol};

#[tokio::test]
async fn proper_borrow_rates() {
    let (contracts, _admin, mut _wallets) = setup_protocol(10, 5).await;

    let borrow_rate = trove_manager_abi::get_borrowing_rate(&contracts.trove_manager)
        .await
        .value;

    // 0.5%
    assert_eq!(borrow_rate, 5_000_000);

    let amount_borrowed = 1_000_000_000;

    let borrow_fee =
        trove_manager_abi::get_borrowing_fee(&contracts.trove_manager, amount_borrowed)
            .await
            .value;

    assert_eq!(borrow_fee, 5_000_000);

    let borrow_rate_with_delay =
        trove_manager_abi::get_borrowing_rate_with_decay(&contracts.trove_manager)
            .await
            .value;

    // 0.5%
    assert_eq!(borrow_rate_with_delay, 5_000_000);
}

#[tokio::test]
async fn proper_redemption_rates() {
    let (contracts, _admin, mut _wallets) = setup_protocol(10, 5).await;

    let redemption_rate = trove_manager_abi::get_redemption_rate(&contracts.trove_manager)
        .await
        .value;

    // 0.5%
    assert_eq!(redemption_rate, 5_000_000);

    let redemption_rate_with_delay =
        trove_manager_abi::get_redemption_rate_with_decay(&contracts.trove_manager)
            .await
            .value;

    // 0.5%
    assert_eq!(redemption_rate_with_delay, 5_000_000);

    let amount_redeemed: u64 = 1_000_000_000;

    let redemption_fee =
        trove_manager_abi::get_redemption_fee_with_decay(&contracts.trove_manager, amount_redeemed)
            .await
            .value;

    assert_eq!(redemption_fee, 5_000_000);
}
