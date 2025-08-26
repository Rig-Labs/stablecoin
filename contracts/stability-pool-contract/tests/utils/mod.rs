use fuels::accounts::{wallet::Wallet, ViewOnlyAccount};
use test_utils::data_structures::ProtocolContracts;

pub mod setup;

pub async fn assert_no_fpt_issued(
    contracts: &ProtocolContracts<Wallet>,
    admin: &Wallet,
    initial_fpt_balance: u64,
) {
    let ProtocolContracts {
        fpt_asset_id,
        community_issuance,
        ..
    } = contracts;

    assert!(
        admin.get_asset_balance(fpt_asset_id).await.unwrap() == initial_fpt_balance,
        "Admin should have the same FPT balance as before"
    );

    community_issuance
        .contract
        .get_balances()
        .await
        .unwrap()
        .keys()
        .for_each(|k| {
            assert!(
                k != fpt_asset_id,
                "Issuance contract should not have FPT tokens"
            )
        });
}
