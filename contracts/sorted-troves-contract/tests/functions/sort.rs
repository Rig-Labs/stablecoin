use fuels::types::Identity;

use crate::utils::setup::setup;

#[tokio::test]
async fn proper_initialization() {
    let (sorted_troves, trove_manager, _wallet, _wallet2) = setup().await;
    let max_size: u64 = 1000;
    // Increment the counter
    let _result = sorted_troves
        .methods()
        .set_params(
            max_size,
            trove_manager.contract_id().into(),
            trove_manager.contract_id().into(),
        )
        .call()
        .await
        .unwrap();

    // Get the current value of the counter
    let result = sorted_troves.methods().get_max_size().call().await.unwrap();

    assert_eq!(result.value, max_size);

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 0);

    let first = sorted_troves.methods().get_first().call().await.unwrap();
    assert_eq!(first.value, Identity::Address([0; 32].into()));

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(last.value, Identity::Address([0; 32].into()));
}
