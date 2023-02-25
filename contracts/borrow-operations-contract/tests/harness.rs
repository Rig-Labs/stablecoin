use fuels::{prelude::*, types::Identity};

// Load abi from json
use test_utils::{
    interfaces::borrow_operations::borrow_operations_abi,
    interfaces::borrow_operations::BorrowOperations,
    interfaces::oracle::oracle_abi,
    interfaces::oracle::Oracle,
    interfaces::sorted_troves as sorted_troves_abi,
    interfaces::sorted_troves::SortedTroves,
    interfaces::token as token_abi,
    interfaces::token::Token,
    interfaces::trove_manager as trove_manager_abi,
    interfaces::trove_manager::TroveManagerContract,
    setup::common::{
        deploy_borrow_operations, deploy_oracle, deploy_sorted_troves, deploy_token,
        deploy_trove_manager_contract,
    },
};

async fn get_contract_instances() -> (
    BorrowOperations,
    TroveManagerContract,
    Oracle,
    SortedTroves,
    Token, /* Fuel */
    Token, /* USDF */
    WalletUnlocked,
) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let bo_instance = deploy_borrow_operations(&wallet).await;
    let oracle_instance = deploy_oracle(&wallet).await;
    let sorted_troves = deploy_sorted_troves(&wallet).await;
    let trove_manger = deploy_trove_manager_contract(&wallet).await;
    let fuel = deploy_token(&wallet).await;
    let usdf = deploy_token(&wallet).await;

    let _ = token_abi::initialize(
        &fuel,
        1_000_000_000,
        &Identity::Address(wallet.address().into()),
        "Fuel".to_string(),
        "FUEL".to_string(),
    )
    .await;

    let _ = token_abi::initialize(
        &usdf,
        0,
        &Identity::ContractId(bo_instance.contract_id().into()),
        "USD Fuel".to_string(),
        "USDF".to_string(),
    )
    .await;

    let _ = sorted_troves_abi::initialize(
        &sorted_troves,
        100,
        bo_instance.contract_id().into(),
        trove_manger.contract_id().into(),
    )
    .await;

    let _ = trove_manager_abi::initialize(&trove_manger, bo_instance.contract_id().into()).await;

    let _ = oracle_abi::set_price(&oracle_instance, 1_000_000).await;

    (
        bo_instance,
        trove_manger,
        oracle_instance,
        sorted_troves,
        fuel,
        usdf,
        wallet,
    )
}

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
    ) = get_contract_instances().await;

    let _ = token_abi::mint_to_id(&fuel_token, 5_000_000_000, &admin).await;

    let fuel_asset_id = AssetId::from(*fuel_token.contract_id().hash());

    let provider = admin.get_provider().unwrap();
    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await;

    println!(
        "Admin FUEL balance Before: {:?}",
        admin_balance.unwrap() / 1_000_000
    );

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
        1_000_000_000,
        600_000_000,
        Identity::Address([0; 32].into()),
        Identity::Address([0; 32].into()),
    )
    .await;

    // println!("{:?}", res);
    let admin_balance = provider
        .get_asset_balance(admin.address().into(), fuel_asset_id)
        .await;

    println!(
        "Admin FUEL balance After Deposit: {:?}",
        admin_balance.unwrap() / 1_000_000
    );

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
    let _icr = trove_manager_abi::get_nominal_icr(
        &trove_manager,
        Identity::Address(admin.address().into()),
    )
    .await
    .value;

    assert_eq!(size, 1);
    assert_eq!(first, Identity::Address(admin.address().into()));
    assert_eq!(last, Identity::Address(admin.address().into()));
    assert_eq!(usdf_balance, 600_000_000);

    println!("Admin USDF balance: {:?}", usdf_balance / 1_000_000);
    // println!("ICR: {:?}", icr);
    // TODO redo ICR calculation in trove_manager
}
