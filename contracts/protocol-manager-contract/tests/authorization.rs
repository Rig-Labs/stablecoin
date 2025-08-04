use test_utils::{
    data_structures::{ContractInstance, ExistingAssetContracts},
    interfaces::protocol_manager::{protocol_manager_abi, ProtocolManager},
    setup::common::{deploy_asset_contracts, initialize_asset, setup_protocol},
};

#[tokio::test]
async fn test_authorizations() {
    let (mut contracts, protocol_manager_owner, mut wallets) =
        setup_protocol(5, false, false).await;

    let attacker = wallets.pop().unwrap();

    // Test 1: Unauthorized renounce_admin
    let protocol_manager_attacker = ContractInstance::new(
        ProtocolManager::new(
            contracts.protocol_manager.contract.contract_id().clone(),
            attacker.clone(),
        ),
        contracts.protocol_manager.implementation_id,
    );

    let result = protocol_manager_abi::renounce_admin(&protocol_manager_attacker).await;

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to renounce admin"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }
    // Test 2: Unauthorized register_asset
    let existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        symbol: "".to_string(),
        asset: None,
        pyth_oracle: None,
        redstone_oracle: None,
    };
    let asset_contracts = deploy_asset_contracts(
        &protocol_manager_owner,
        &existing_asset_to_initialize,
        true,
        true,
    )
    .await;
    contracts.protocol_manager = protocol_manager_attacker.clone();
    let result = initialize_asset(&contracts, &asset_contracts).await;

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to initialize an asset"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let protocol_manager_owner_contract = ContractInstance::new(
        ProtocolManager::new(
            contracts.protocol_manager.contract.contract_id().clone(),
            protocol_manager_owner.clone(),
        ),
        contracts.protocol_manager.implementation_id,
    );
    // Test 3: Authorized register_asset
    contracts.protocol_manager = protocol_manager_owner_contract.clone();

    let existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        symbol: "".to_string(),
        asset: None,
        pyth_oracle: None,
        redstone_oracle: None,
    };

    let asset_contracts_owner = deploy_asset_contracts(
        &protocol_manager_owner,
        &existing_asset_to_initialize,
        true,
        true,
    )
    .await;
    let result = initialize_asset(&contracts, &asset_contracts_owner).await;

    assert!(
        result.is_ok(),
        "Authorized user should be able to initialize an asset"
    );

    // Test 4: Duplicate asset registration
    let result = protocol_manager_abi::register_asset(
        &protocol_manager_owner_contract,
        asset_contracts_owner.asset_id,
        asset_contracts_owner
            .trove_manager
            .contract
            .contract_id()
            .into(),
        asset_contracts_owner.oracle.contract.contract_id().into(),
        &contracts.borrow_operations,
        &contracts.stability_pool,
        &contracts.usdm,
        &contracts.fpt_staking,
        &contracts.coll_surplus_pool,
        &contracts.default_pool,
        &contracts.active_pool,
        &contracts.sorted_troves,
    )
    .await;

    assert!(result.is_err(), "Duplicate asset registration should fail");

    // Test unauthorized transfer
    let result = protocol_manager_abi::transfer_owner(
        &protocol_manager_attacker,
        fuels::types::Identity::Address(attacker.address().into()),
    )
    .await;
    assert!(
        result.is_err(),
        "Unauthorized user should not be able to transfer ownership"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    // Test authorized transfer
    let new_owner = wallets.pop().unwrap();
    let result = protocol_manager_abi::transfer_owner(
        &protocol_manager_owner_contract,
        fuels::types::Identity::Address(new_owner.address().into()),
    )
    .await;
    assert!(
        result.is_ok(),
        "Authorized user should be able to transfer ownership"
    );

    // Verify old owner can't perform admin actions
    let existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        symbol: "".to_string(),
        asset: None,
        pyth_oracle: None,
        redstone_oracle: None,
    };
    let asset_contracts = deploy_asset_contracts(
        &protocol_manager_owner,
        &existing_asset_to_initialize,
        true,
        true,
    )
    .await;
    let result = initialize_asset(&contracts, &asset_contracts).await;
    assert!(
        result.is_err(),
        "Old owner should not be able to initialize an asset after transfer"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let new_protocol_manager_owner = ContractInstance::new(
        ProtocolManager::new(
            contracts.protocol_manager.contract.contract_id().clone(),
            new_owner.clone(),
        ),
        contracts.protocol_manager.implementation_id,
    );
    // Test 5: Authorized renounce_admin
    let result = protocol_manager_abi::renounce_admin(&new_protocol_manager_owner).await;

    assert!(
        result.is_ok(),
        "Authorized user should be able to renounce admin"
    );

    // Test 6: Unauthorized register_asset after renouncement
    let existing_asset_to_initialize: ExistingAssetContracts = ExistingAssetContracts {
        symbol: "".to_string(),
        asset: None,
        pyth_oracle: None,
        redstone_oracle: None,
    };
    let unauthorized_asset_contracts = deploy_asset_contracts(
        &protocol_manager_owner,
        &existing_asset_to_initialize,
        true,
        true,
    )
    .await;
    let result = initialize_asset(&contracts, &unauthorized_asset_contracts).await;

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to initialize an asset"
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }
}
