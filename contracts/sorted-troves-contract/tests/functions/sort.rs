use fuels::accounts::fuel_crypto::rand::{self, Rng};
use fuels::{prelude::TxParameters, types::Identity};

use crate::utils::setup::{initialize_st_and_tm, remove, set_nominal_icr_and_insert, setup};

use crate::utils::sorted_troves::sorted_troves_utils::{
    assert_in_order_from_head, assert_in_order_from_tail, assert_neighbors, generate_random_nodes,
};

use test_utils::interfaces::sorted_troves::sorted_troves_abi;

#[tokio::test]
async fn proper_initialization() {
    let (sorted_troves, trove_manager, _wallet, _wallet2, _) = setup(Some(4)).await;
    let max_size: u64 = 1000;
    // Increment the counter
    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    // Get the current value of the counter
    let result = sorted_troves.methods().get_max_size().call().await.unwrap();

    assert_eq!(result.value, max_size);

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 0);

    let first = sorted_troves_abi::get_first(&sorted_troves).await;
    assert_eq!(first.value, Identity::Address([0; 32].into()));

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(last.value, Identity::Address([0; 32].into()));
}

#[tokio::test]
async fn proper_head_and_tails_after_insert() {
    let (sorted_troves, trove_manager, wallet, wallet2, _) = setup(Some(4)).await;
    let max_size: u64 = 1000;
    // Increment the counter
    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    // Get the current value of the counter
    // check if contains
    let result = sorted_troves
        .methods()
        .contains(Identity::Address(wallet.address().into()))
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, false);

    let tx_params = TxParameters::default().set_gas_price(1);

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
        ),
        "Empty list should return 0, 0 placements"
    );

    let _ = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(wallet.address().into()),
        100,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // State:
    // (100, wallet)

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 1);

    let first = sorted_troves_abi::get_first(&sorted_troves).await;
    assert_eq!(
        first.value,
        Identity::Address(wallet.address().into()),
        "first should be wallet"
    );

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(last.value, Identity::Address(wallet.address().into()));

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(wallet2.address().into()),
        200,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // State:
    // (200, wallet2) -> (100, wallet)

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 2);

    let first = sorted_troves_abi::get_first(&sorted_troves).await;
    assert_eq!(
        first.value,
        Identity::Address(wallet2.address().into()),
        "First should be wallet2"
    );

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(
        last.value,
        Identity::Address(wallet.address().into()),
        "Last should be wallet"
    );

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::ContractId(trove_manager.contract_id().into()),
        300,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // State:
    // (300, trove_manager) -> (200, wallet2) -> (100, wallet)

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 3);

    let first = sorted_troves_abi::get_first(&sorted_troves).await;
    assert_eq!(
        first.value,
        Identity::ContractId(trove_manager.contract_id().into()),
        "First should be trove manager"
    );

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(
        last.value,
        Identity::Address(wallet.address().into()),
        "Last should be wallet"
    );

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::ContractId(sorted_troves.contract_id().into()),
        150,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;
    // State:
    // (300, trove_manager) -> (200, wallet2) -> (150, sorted_troves) -> (100, wallet)

    let result_size = sorted_troves.methods().get_size().call().await.unwrap();
    assert_eq!(result_size.value, 4);

    let first = sorted_troves_abi::get_first(&sorted_troves).await;

    assert_eq!(
        first.value,
        Identity::ContractId(trove_manager.contract_id().into()),
        "First should be trove manager"
    );

    let last = sorted_troves.methods().get_last().call().await.unwrap();
    assert_eq!(
        last.value,
        Identity::Address(wallet.address().into()),
        "Last should be wallet"
    );
}

#[tokio::test]
async fn proper_node_neighbors() {
    let (sorted_troves, trove_manager, wallet, wallet2, _) = setup(Some(4)).await;
    let max_size: u64 = 1000;
    // Increment the counter
    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    let _ = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(wallet.address().into()),
        100,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // Current state
    // wallet: 100

    let _ = assert_neighbors(
        &sorted_troves,
        Identity::Address(wallet.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(wallet2.address().into()),
        200,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // Current state
    // wallet2: 200 -> wallet: 100

    let _ = assert_neighbors(
        &sorted_troves,
        Identity::Address(wallet2.address().into()),
        Identity::Address([0; 32].into()),
        Identity::Address(wallet2.address().into()),
    );

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::ContractId(trove_manager.contract_id().into()),
        300,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // Current state
    // trove_manager: 300 -> wallet2: 200 -> wallet: 100

    let _ = assert_neighbors(
        &sorted_troves,
        Identity::ContractId(trove_manager.contract_id().into()),
        Identity::Address([0; 32].into()),
        Identity::Address(wallet2.address().into()),
    );

    let _res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::ContractId(sorted_troves.contract_id().into()),
        150,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // Current state
    // trove_manager: 300 -> wallet2: 200 -> sorted_troves: 150 -> wallet: 100

    let _ = assert_neighbors(
        &sorted_troves,
        Identity::ContractId(sorted_troves.contract_id().into()),
        Identity::Address(wallet2.address().into()),
        Identity::Address(wallet.address().into()),
    );
}

#[tokio::test]
async fn proper_insertion_of_random_nodes() {
    let max_size: u64 = 10;
    let (sorted_troves, trove_manager, _, _, _) = setup(Some(4)).await;

    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    let _ = generate_random_nodes(&trove_manager, &sorted_troves, max_size).await;

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;

    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;
}

#[tokio::test]
async fn proper_hint_gas_usage() {
    let max_size: u64 = 20;
    let (sorted_troves, trove_manager, _, _, _) = setup(Some(4)).await;

    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    let (mut vals, avg_gas) = generate_random_nodes(&trove_manager, &sorted_troves, 15).await;

    vals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut rng = rand::thread_rng();
    let random_address = rng.gen::<[u8; 32]>();

    let inbetween_num = vals[6].1 + (vals[5].1 - vals[6].1) / 2;

    let res = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(random_address.into()),
        inbetween_num,
        vals[5].0.clone(),
        vals[6].0.clone(),
    )
    .await;

    let gas_with_hint = res.gas_used;
    assert!(gas_with_hint < avg_gas);

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;
    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;

    // only use prev hint
    let random_addr_2 = rng.gen::<[u8; 32]>();

    let inbetween_num2 = vals[9].1 + (vals[8].1 - vals[9].1) / 2;

    let res2 = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(random_addr_2.into()),
        inbetween_num2,
        vals[8].0.clone(),
        Identity::Address([0; 32].into()),
    )
    .await;
    let gas_prev_hint = res2.gas_used;
    assert!(gas_prev_hint < avg_gas);

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;
    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;

    // only use next hint
    let random_addr_3 = rng.gen::<[u8; 32]>();

    let inbetween_num3 = vals[12].1 + 1;

    let res3 = set_nominal_icr_and_insert(
        &trove_manager,
        &sorted_troves,
        Identity::Address(random_addr_3.into()),
        inbetween_num3,
        Identity::Address([0; 32].into()),
        vals[12].0.clone(),
    )
    .await;
    let gas_next_hint = res3.gas_used;
    assert!(
        gas_next_hint < avg_gas,
        "average gas: {}, gas used: {}",
        avg_gas,
        gas_next_hint
    );

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;
    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;
}

#[tokio::test]
async fn proper_removal() {
    let max_size: u64 = 10;
    let (sorted_troves, trove_manager, _, _, _) = setup(Some(4)).await;

    let _ = initialize_st_and_tm(&sorted_troves, &trove_manager, max_size).await;

    let (mut nodes, _) = generate_random_nodes(&trove_manager, &sorted_troves, max_size).await;

    // get random node
    let rand_node = nodes.pop().unwrap();

    let _res = remove(&trove_manager, &sorted_troves, rand_node.0).await;

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;

    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;

    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;

    assert_eq!(size, max_size - 1);

    let rand_node = nodes.pop().unwrap();

    let _res = remove(&trove_manager, &sorted_troves, rand_node.0).await;
    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;

    assert_eq!(size, max_size - 2);

    let _ = assert_in_order_from_head(&sorted_troves, &trove_manager).await;

    let _ = assert_in_order_from_tail(&sorted_troves, &trove_manager).await;
}
