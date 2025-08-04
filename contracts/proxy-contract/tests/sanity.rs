use fuels::prelude::*;

use test_utils::{
    interfaces::proxy::{proxy_abi, Proxy},
    setup::common::setup_protocol,
};

#[tokio::test]
async fn sanity_testing_setup() {
    // Setup protocol with initial contracts
    let (contracts, admin, mut other) = setup_protocol(2, false, false).await;

    let attacker = other.pop().unwrap();
    // Try to upgrade each contract with unauthorized wallet
    let new_target = ContractId::from([3u8; 32]);

    let attacker_proxy_usdm = Proxy::new(
        contracts.usdm.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test USDM token proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_usdm, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade USDM token"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_usdm = Proxy::new(contracts.usdm.contract.contract_id().clone(), admin.clone());
    // Verify admin can still upgrade (testing one contract as example)
    let res = proxy_abi::set_proxy_target(&admin_proxy_usdm, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_borrow_operations = Proxy::new(
        contracts.borrow_operations.contract.contract_id().clone(),
        attacker.clone(),
    );

    // Test Borrow Operations proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_borrow_operations, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Borrow Operations"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    // verify admin can upgrade
    let admin_proxy_sorted_troves = Proxy::new(
        contracts.sorted_troves.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_sorted_troves, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_sorted_troves = Proxy::new(
        contracts.sorted_troves.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Sorted Troves proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_sorted_troves, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Sorted Troves"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_active_pool = Proxy::new(
        contracts.active_pool.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_active_pool, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_active_pool = Proxy::new(
        contracts.active_pool.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Active Pool proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_active_pool, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Active Pool"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_default_pool = Proxy::new(
        contracts.default_pool.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_default_pool, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_default_pool = Proxy::new(
        contracts.default_pool.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Default Pool proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_default_pool, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Default Pool"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_fpt_staking = Proxy::new(
        contracts.fpt_staking.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_fpt_staking, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_fpt_staking = Proxy::new(
        contracts.fpt_staking.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test FPT Staking proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_fpt_staking, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade FPT Staking"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_fpt_token = Proxy::new(
        contracts.fpt_token.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_fpt_token, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    // Test each asset contract's components
    for asset_contract in &contracts.asset_contracts {
        let attacker_proxy_trove_manager = Proxy::new(
            asset_contract.trove_manager.contract.contract_id().clone(),
            attacker.clone(),
        );
        // Test Trove Manager proxy
        let res = proxy_abi::set_proxy_target(&attacker_proxy_trove_manager, new_target).await;
        assert!(
            res.is_err(),
            "Unauthorized wallet should not be able to upgrade Trove Manager"
        );
        if let Err(error) = res {
            assert!(
                error.to_string().contains("NotOwner"),
                "Unexpected error message: {}",
                error
            );
        }

        let admin_proxy_oracle = Proxy::new(
            asset_contract.oracle.contract.contract_id().clone(),
            admin.clone(),
        );
        let res = proxy_abi::set_proxy_target(&admin_proxy_oracle, new_target).await;
        assert!(res.is_ok(), "Admin should be able to upgrade contracts");

        let attacker_proxy_oracle = Proxy::new(
            asset_contract.oracle.contract.contract_id().clone(),
            attacker.clone(),
        );
        // Test Oracle proxy
        let res = proxy_abi::set_proxy_target(&attacker_proxy_oracle, new_target).await;
        assert!(
            res.is_err(),
            "Unauthorized wallet should not be able to upgrade Oracle"
        );
        if let Err(error) = res {
            assert!(
                error.to_string().contains("NotOwner"),
                "Unexpected error message: {}",
                error
            );
        }

        let admin_proxy_oracle = Proxy::new(
            asset_contract.oracle.contract.contract_id().clone(),
            admin.clone(),
        );
        let res = proxy_abi::set_proxy_target(&admin_proxy_oracle, new_target).await;
        assert!(res.is_ok(), "Admin should be able to upgrade contracts");
    }

    let admin_proxy_stability_pool = Proxy::new(
        contracts.stability_pool.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_stability_pool, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_stability_pool = Proxy::new(
        contracts.stability_pool.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Stability Pool proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_stability_pool, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Stability Pool"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let attacker_proxy_coll_surplus_pool = Proxy::new(
        contracts.coll_surplus_pool.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Collateral Surplus Pool proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_coll_surplus_pool, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Collateral Surplus Pool"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_community_issuance = Proxy::new(
        contracts.community_issuance.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_community_issuance, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_community_issuance = Proxy::new(
        contracts.community_issuance.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Community Issuance proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_community_issuance, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Community Issuance"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_vesting = Proxy::new(
        contracts.vesting_contract.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_vesting, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_vesting = Proxy::new(
        contracts.vesting_contract.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Vesting proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_vesting, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Vesting"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_protocol_manager = Proxy::new(
        contracts.protocol_manager.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_protocol_manager, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_protocol_manager = Proxy::new(
        contracts.protocol_manager.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test Protocol Manager proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_protocol_manager, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade Protocol Manager"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }

    let admin_proxy_fpt_token = Proxy::new(
        contracts.fpt_token.contract.contract_id().clone(),
        admin.clone(),
    );
    let res = proxy_abi::set_proxy_target(&admin_proxy_fpt_token, new_target).await;
    assert!(res.is_ok(), "Admin should be able to upgrade contracts");

    let attacker_proxy_fpt_token = Proxy::new(
        contracts.fpt_token.contract.contract_id().clone(),
        attacker.clone(),
    );
    // Test FPT Token proxy
    let res = proxy_abi::set_proxy_target(&attacker_proxy_fpt_token, new_target).await;
    assert!(
        res.is_err(),
        "Unauthorized wallet should not be able to upgrade FPT Token"
    );
    if let Err(error) = res {
        assert!(
            error.to_string().contains("NotOwner"),
            "Unexpected error message: {}",
            error
        );
    }
}
