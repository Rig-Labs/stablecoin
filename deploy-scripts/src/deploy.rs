use std::{fs::File, io::Write};

use dotenv::dotenv;
use fuels::prelude::*;
use serde_json::json;
use test_utils::interfaces::{
    community_issuance::community_issuance_abi, fpt_token::fpt_token_abi,
};

pub mod deployment {
    const VESTING_SCHEDULE_PATH: &str = "deploy-scripts/vesting/test_vesting.json";
    use fuels::types::Identity;
    use pbr::ProgressBar;
    use test_utils::interfaces::vesting::{self, load_vesting_schedules_from_json_file};

    use super::*;

    use test_utils::interfaces::{
        active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
        coll_surplus_pool::coll_surplus_pool_abi, default_pool::default_pool_abi,
        fpt_staking::fpt_staking_abi, protocol_manager::protocol_manager_abi,
        sorted_troves::sorted_troves_abi, stability_pool::stability_pool_abi,
        usdf_token::usdf_token_abi,
    };
    use test_utils::setup::common::{
        deploy_active_pool, deploy_borrow_operations, deploy_coll_surplus_pool,
        deploy_community_issuance, deploy_default_pool, deploy_fpt_staking, deploy_fpt_token,
        deploy_protocol_manager, deploy_sorted_troves, deploy_stability_pool, deploy_token,
        deploy_usdf_token, deploy_vesting_contract, AssetContracts, ProtocolContracts,
    };

    pub async fn deploy() {
        //--------------- Deploy ---------------
        dotenv().ok();

        //--------------- WALLET ---------------
        let wallet = setup_wallet().await;
        let address = wallet.address();
        println!("🔑 Wallet address: {}", address);

        //--------------- Deploy ---------------
        // TODO: Figure out max size
        let core_contracts =
            deployment::deploy_and_initialize_all_core_contracts(wallet, 100_000).await;

        //--------------- Write to file ---------------
        write_contracts_to_file(core_contracts)
    }

    pub async fn deploy_and_initialize_all_core_contracts(
        wallet: WalletUnlocked,
        max_size: u64,
    ) -> ProtocolContracts<WalletUnlocked> {
        println!("Deploying core contracts...");
        let mut pb = ProgressBar::new(13);

        let borrow_operations = deploy_borrow_operations(&wallet).await;
        pb.inc();

        let usdf = deploy_usdf_token(&wallet).await;
        pb.inc();

        let fpt_token = deploy_fpt_token(&wallet).await;
        pb.inc();

        let fpt_staking = deploy_fpt_staking(&wallet).await;
        pb.inc();

        let stability_pool = deploy_stability_pool(&wallet).await;
        pb.inc();

        let protocol_manager = deploy_protocol_manager(&wallet).await;
        pb.inc();

        let community_issuance = deploy_community_issuance(&wallet).await;
        pb.inc();

        let coll_surplus_pool = deploy_coll_surplus_pool(&wallet).await;
        pb.inc();

        let default_pool = deploy_default_pool(&wallet).await;
        pb.inc();

        let active_pool = deploy_active_pool(&wallet).await;
        pb.inc();

        let sorted_troves = deploy_sorted_troves(&wallet).await;
        pb.inc();

        let vesting_contract = deploy_vesting_contract(&wallet).await;
        pb.inc();

        println!("Borrow operations: {}", borrow_operations.contract_id());
        println!("USDF Token: {}", usdf.contract_id());
        println!("Stability Pool: {}", stability_pool.contract_id());
        println!("FPT Staking: {}", fpt_staking.contract_id());
        println!("FPT Token: {}", fpt_token.contract_id());
        println!("Community Issuance {}", community_issuance.contract_id());
        println!("Coll Surplus Pool {}", coll_surplus_pool.contract_id());
        println!("Protocol Manager {}", protocol_manager.contract_id());
        println!("Default Pool {}", default_pool.contract_id());
        println!("Active Pool {}", active_pool.contract_id());
        println!("Sorted Troves {}", sorted_troves.contract_id());
        println!("Initializing contracts...");

        let mut pb = ProgressBar::new(8);

        let asset_contracts: Vec<AssetContracts<WalletUnlocked>> = vec![];
        wait();

        let fpt_token_id: AssetId = fpt_token.contract_id().asset_id(&AssetId::zeroed().into());

        let _ = community_issuance_abi::initialize(
            &community_issuance,
            stability_pool.contract_id().into(),
            fpt_token
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            &Identity::Address(wallet.address().into()),
            false,
        )
        .await;
        pb.inc();

        let _ = fpt_token_abi::initialize(&fpt_token, &vesting_contract, &community_issuance).await;
        pb.inc();

        let _ = usdf_token_abi::initialize(
            &usdf,
            protocol_manager.contract_id().into(),
            Identity::ContractId(stability_pool.contract_id().into()),
            Identity::ContractId(borrow_operations.contract_id().into()),
        )
        .await;
        pb.inc();

        let _ = borrow_operations_abi::initialize(
            &borrow_operations,
            usdf.contract_id().into(),
            fpt_staking.contract_id().into(),
            protocol_manager.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            active_pool.contract_id().into(),
            sorted_troves.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = stability_pool_abi::initialize(
            &stability_pool,
            usdf.contract_id().into(),
            community_issuance.contract_id().into(),
            protocol_manager.contract_id().into(),
            active_pool.contract_id().into(),
        )
        .await
        .unwrap();
        wait();
        pb.inc();

        let _ = fpt_staking_abi::initialize(
            &fpt_staking,
            protocol_manager.contract_id().into(),
            borrow_operations.contract_id().into(),
            fpt_token_id,
            usdf.contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = coll_surplus_pool_abi::initialize(
            &coll_surplus_pool,
            borrow_operations.contract_id().into(),
            Identity::ContractId(protocol_manager.contract_id().into()),
        )
        .await;
        wait();
        pb.inc();

        let _ = protocol_manager_abi::initialize(
            &protocol_manager,
            borrow_operations.contract_id().into(),
            stability_pool.contract_id().into(),
            fpt_staking.contract_id().into(),
            usdf.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            default_pool.contract_id().into(),
            active_pool.contract_id().into(),
            sorted_troves.contract_id().into(),
            Identity::Address(wallet.address().into()),
        )
        .await;
        wait();
        pb.inc();

        let _ = default_pool_abi::initialize(
            &default_pool,
            Identity::ContractId(protocol_manager.contract_id().into()),
            active_pool.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = active_pool_abi::initialize(
            &active_pool,
            Identity::ContractId(borrow_operations.contract_id().into()),
            Identity::ContractId(stability_pool.contract_id().into()),
            default_pool.contract_id().into(),
            Identity::ContractId(protocol_manager.contract_id().into()),
        )
        .await;
        wait();
        pb.inc();

        // TODO: Verify max size is correct
        let _ = sorted_troves_abi::initialize(
            &sorted_troves,
            max_size,
            protocol_manager.contract_id().into(),
            borrow_operations.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let vesting_schedules = load_vesting_schedules_from_json_file(VESTING_SCHEDULE_PATH);
        let _ = vesting::instantiate_vesting_contract(
            &vesting_contract,
            &fpt_token_id,
            vesting_schedules,
        )
        .await;

        pb.finish();

        let contracts = ProtocolContracts {
            borrow_operations,
            usdf,
            stability_pool,
            protocol_manager,
            asset_contracts,
            fpt_staking,
            fpt_token,
            community_issuance,
            coll_surplus_pool,
            default_pool,
            sorted_troves,
            active_pool,
            vesting_contract,
        };

        return contracts;
    }

    pub async fn setup_wallet() -> WalletUnlocked {
        let rpc = match std::env::var("RPC") {
            Ok(s) => s,
            Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
        };
        println!("RPC: {}", rpc);

        let provider = match Provider::connect(rpc).await {
            Ok(p) => p,
            Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
        };

        let secret = match std::env::var("SECRET") {
            Ok(s) => s,
            Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
        };

        WalletUnlocked::new_from_mnemonic_phrase_with_path(
            &secret,
            Some(provider),
            "m/44'/1179993420'/0'/0/0",
        )
        .unwrap()
    }

    fn write_contracts_to_file(contracts: ProtocolContracts<WalletUnlocked>) {
        let mut file = File::create("contracts.json").unwrap();

        let json = json!({
            "borrow_operations": contracts.borrow_operations.contract_id().to_string(),
            "usdf": contracts.usdf.contract_id().to_string(),
            "usdf_asset_id": contracts.usdf.contract_id().asset_id(&AssetId::zeroed().into()).to_string(),
            "stability_pool": contracts.stability_pool.contract_id().to_string(),
            "protocol_manager": contracts.protocol_manager.contract_id().to_string(),
            "fpt_staking": contracts.fpt_staking.contract_id().to_string(),
            "fpt_token": contracts.fpt_token.contract_id().to_string(),
            "fpt_asset_id": contracts.fpt_token.contract_id().asset_id(&AssetId::zeroed().into()).to_string(),
            "community_issuance": contracts.community_issuance.contract_id().to_string(),
            "coll_surplus_pool": contracts.coll_surplus_pool.contract_id().to_string(),
            "default_pool": contracts.default_pool.contract_id().to_string(),
            "active_pool": contracts.active_pool.contract_id().to_string(),
            "sorted_troves": contracts.sorted_troves.contract_id().to_string(),
            "vesting_contract": contracts.vesting_contract.contract_id().to_string(),
            "asset_contracts" : contracts.asset_contracts.iter().map(|asset_contracts| {
                json!({
                    "oracle": asset_contracts.oracle.contract_id().to_string(),
                    "trove_manager": asset_contracts.trove_manager.contract_id().to_string(),
                    "asset_contract": asset_contracts.asset.contract_id().to_string(),
                    "asset_id": asset_contracts.asset_id.to_string(),
                })
            }).collect::<Vec<serde_json::Value>>()
        });

        file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
            .unwrap();
    }

    pub fn wait() {
        // Necessary for random instances where the 'UTXO' cannot be found
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}
