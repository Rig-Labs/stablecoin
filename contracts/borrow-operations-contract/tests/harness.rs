use fuels::{prelude::*, types::Identity};

// Load abi from json
use test_utils::{
    interfaces::borrow_operations::borrow_operations_abi,
    interfaces::sorted_troves::sorted_troves_abi, interfaces::token::token_abi,
    interfaces::trove_manager::trove_manager_abi, setup::common::setup_protocol,
};

#[tokio::test]
async fn proper_creating_trove() {
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        admin,
    ) = setup_protocol(100).await;

    let _ = token_abi::mint_to_id(&fuel_token, 5_000_000_000, &admin).await;

    // let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

    let provider = admin.get_provider().unwrap();
    // let admin_balance = provider
    //     .get_asset_balance(admin.address().into(), fuel_asset_id)
    //     .await;

    // println!(
    //     "Admin FUEL balance Before: {:?}",
    //     admin_balance.unwrap() / 1_000_000
    // );

    // let bo_instance = BorrowOperations::new(borrow_operations_instance.id().clone(), admin);

    let _ = borrow_operations_abi::initialize(
        &borrow_operations_instance,
        trove_manager.contract_id().into(),
        sorted_troves.contract_id().into(),
        oracle.contract_id().into(),
        fuel_token.contract_id().into(),
        usdf_token.contract_id().into(),
        usdf_token.contract_id().into(),
    )
    .await;

    let _ = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        0,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // println!("{:?}", res);
    // let admin_balance = provider
    //     .get_asset_balance(admin.address().into(), fuel_asset_id)
    //     .await;

    // println!(
    //     "Admin FUEL balance After Deposit: {:?}",
    //     admin_balance.unwrap() / 1_000_000
    // );

    // check usdf balance
    let usdf_balance = provider
        .get_asset_balance(
            admin.address().into(),
            AssetId::from(*usdf_token.contract_id().hash()),
        )
        .await
        .unwrap();

    let first = sorted_troves_abi::get_first(&sorted_troves).await.value;
    let last = sorted_troves_abi::get_last(&sorted_troves).await.value;
    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;
    let icr = trove_manager_abi::get_nominal_icr(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 600_000_000);

    // println!("Admin USDF balance: {:?}", usdf_balance / 1_000_000);
    // println!("ICR: {:?}", icr);
    assert_eq!(icr, 2_000_000_000, "ICR is wrong");

    let trove_col = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 1_200_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, 600_000_000, "Trove Debt is wrong");
    // TODO redo ICR calculation in trove_manager
}

#[tokio::test]
async fn proper_increase_collateral() {
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        admin,
    ) = setup_protocol(100).await;

    let _ = token_abi::mint_to_id(&fuel_token, 5_000_000_000, &admin).await;

    let _ = borrow_operations_abi::initialize(
        &borrow_operations_instance,
        trove_manager.contract_id().into(),
        sorted_troves.contract_id().into(),
        oracle.contract_id().into(),
        fuel_token.contract_id().into(),
        usdf_token.contract_id().into(),
        usdf_token.contract_id().into(),
    )
    .await;

    let _ = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        0,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    borrow_operations_abi::add_coll(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &sorted_troves,
        &trove_manager,
        1_200_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    let trove_col = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 2_400_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, 600_000_000, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&sorted_troves).await.value;
    let last = sorted_troves_abi::get_last(&sorted_troves).await.value;
    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;
    let icr = trove_manager_abi::get_nominal_icr(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, 4_000_000_000, "ICR is wrong");
}

#[tokio::test]
async fn proper_decrease_collateral() {
    let (
        borrow_operations_instance,
        trove_manager,
        oracle,
        sorted_troves,
        fuel_token,
        usdf_token,
        admin,
    ) = setup_protocol(100).await;

    let balance = 5_000_000_000;
    let _ = token_abi::mint_to_id(&fuel_token, balance, &admin).await;

    let _ = borrow_operations_abi::initialize(
        &borrow_operations_instance,
        trove_manager.contract_id().into(),
        sorted_troves.contract_id().into(),
        oracle.contract_id().into(),
        fuel_token.contract_id().into(),
        usdf_token.contract_id().into(),
        usdf_token.contract_id().into(),
    )
    .await;

    let provider = admin.get_provider().unwrap();

    let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

    let _ = borrow_operations_abi::open_trove(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &usdf_token,
        &sorted_troves,
        &trove_manager,
        0,
        1_200_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    borrow_operations_abi::withdraw_coll(
        &borrow_operations_instance,
        &oracle,
        &fuel_token,
        &sorted_troves,
        &trove_manager,
        30_000_0000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    let trove_col = trove_manager_abi::get_trove_coll(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    let trove_debt = trove_manager_abi::get_trove_debt(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(trove_col, 900_000_000, "Trove Collateral is wrong");
    assert_eq!(trove_debt, 600_000_000, "Trove Debt is wrong");

    let first = sorted_troves_abi::get_first(&sorted_troves).await.value;
    let last = sorted_troves_abi::get_last(&sorted_troves).await.value;
    let size = sorted_troves_abi::get_size(&sorted_troves).await.value;
    let icr = trove_manager_abi::get_nominal_icr(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));

    assert_eq!(icr, 1_500_000_000, "ICR is wrong");

    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await
        .unwrap();

    assert_eq!(admin_balance, 4_100_000_000, "Balance is wrong");
}
