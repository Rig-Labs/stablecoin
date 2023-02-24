use fuels::prelude::*;

use test_utils::interfaces::sorted_troves::{initialize, SortedTroves};
use test_utils::interfaces::trove_manager::{initialize as init_tm, TroveManagerContract};
use test_utils::setup::common::{deploy_sorted_troves, deploy_trove_manager_contract};
// TODO: do setup instead of copy/pasted code with minor adjustments

pub async fn setup(
    num_wallets: Option<u64>,
) -> (
    SortedTroves,
    TroveManagerContract,
    WalletUnlocked,
    WalletUnlocked,
    Vec<WalletUnlocked>,
) {
    // Launch a local network and deploy the contract
    let config = Config {
        manual_blocks_enabled: true, // Necessary so the `produce_blocks` API can be used locally
        ..Config::local_node()
    };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            num_wallets,         /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        Some(config),
        None,
    )
    .await;

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();
    let wallet3 = wallets.pop().unwrap();
    let wallet4 = wallets.pop().unwrap();

    let st_instance = deploy_sorted_troves(&wallet).await;
    let trove_instance = deploy_trove_manager_contract(&wallet2).await;

    (st_instance, trove_instance, wallet3, wallet4, wallets)
}

pub async fn initialize_st_and_tm(
    sorted_troves: &SortedTroves,
    trove_manager: &TroveManagerContract,
    max_size: u64,
) {
    let _ = initialize(
        sorted_troves,
        max_size,
        trove_manager.contract_id().into(),
        trove_manager.contract_id().into(),
    )
    .await;

    let _ = init_tm(trove_manager, sorted_troves.contract_id().into()).await;
}
