use fuels::{prelude::TxParameters, types::Identity};

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

    let _result = trove_manager
        .methods()
        .initialize(sorted_troves.contract_id().into())
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

#[tokio::test]
async fn proper_insert() {
    let (sorted_troves, trove_manager, wallet, _wallet2) = setup().await;
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

    let _result = trove_manager
        .methods()
        .initialize(sorted_troves.contract_id().into())
        .call()
        .await
        .unwrap();

    let _ = trove_manager
        .methods()
        .set_nominal_icr(Identity::Address([0; 32].into()), 0)
        .call()
        .await
        .unwrap();

    let res = trove_manager
        .methods()
        .get_nominal_icr(Identity::Address([0; 32].into()))
        .call()
        .await
        .unwrap();

    assert_eq!(res.value, 0);
    // Get the current value of the counter
    // check if contains
    let result = sorted_troves
        .methods()
        .contains(Identity::Address(wallet.address().into()))
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, false);

    let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

    let result = sorted_troves
        .methods()
        .find_insert_position(
            100,
            Identity::Address([0; 32].into()),
            Identity::Address([0; 32].into()),
        )
        .set_contracts(&[&trove_manager])
        .tx_params(tx_params)
        .simulate()
        .await
        .unwrap();

    assert_eq!(
        result.value,
        (
            Identity::Address([0; 32].into()),
            Identity::Address([0; 32].into())
        )
    );

    let result = sorted_troves.methods().get_max_size().call().await.unwrap();

    assert_eq!(result.value, max_size);

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 0);

    let first = sorted_troves.methods().get_first().call().await.unwrap();
    assert_eq!(first.value, Identity::Address([0; 32].into()));

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(last.value, Identity::Address([0; 32].into()));
}
