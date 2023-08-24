use std::{fs::File, io::Write, str::FromStr};

use crate::setup::common::{ExistingAssetContracts, ProtocolContracts};
use dotenv::dotenv;
use fuels::{
    prelude::{Address, Bech32ContractId, Provider, WalletUnlocked},
    types::ContractId,
};
use serde_json::json;

// const RPC: &str = "http://localhost:4000";

// #[tokio::test]
pub async fn deploy() {
    const RPC: &str = "beta-3.fuel.network";
    //--------------- WALLET ---------------
    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let secret = match std::env::var("SECRET") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };

    let wallet = WalletUnlocked::new_from_mnemonic_phrase_with_path(
        &secret,
        Some(provider.clone()),
        "m/44'/1179993420'/0'/0/0",
    )
    .unwrap();

    let address = Address::from(wallet.address());
    println!("🔑 Wallet address: {}", address);

    let eth_contracts = ExistingAssetContracts {
        asset: ContractId::from(
            Bech32ContractId::from_str(
                "fuel17unetj5y6ypk354m5jqt3vtl0z9n68ezftpe8st0krte64ttzlssxqfx0t",
            )
            .unwrap(),
        ),
        oracle: ContractId::from(
            Bech32ContractId::from_str(
                "fuel1mz3e23uzlttn2crmf5zwst6c55lzemtv5pve55v4y9e06h06p4ws94a847",
            )
            .unwrap(),
        ),
    };

    let st_eth_contracts = ExistingAssetContracts {
        asset: ContractId::from(
            Bech32ContractId::from_str(
                "fuel18acrkuvrh4h00g0drgd9xvtr0f9lrqn96k03p83afgrnqh9vmhustqn8em",
            )
            .unwrap(),
        ),
        oracle: ContractId::from(
            Bech32ContractId::from_str(
                "fuel1u4qqr558fx64m68w6f5n23puuz770e9kthxw42yzqrsc095md8dskn5dcj",
            )
            .unwrap(),
        ),
    };

    let contracts: ProtocolContracts<WalletUnlocked> = deployment::deploy_and_initialize_all(
        wallet,
        100,
        true,
        Some(eth_contracts),
        Some(st_eth_contracts),
    )
    .await;

    // Create json with contract addresses
    let mut file = File::create("contracts.json").unwrap();

    let json = json!({
        "borrow_operations": contracts.borrow_operations.contract_id().to_string(),
        "usdf": contracts.usdf.contract_id().to_string(),
        "stability_pool": contracts.stability_pool.contract_id().to_string(),
        "protocol_manager": contracts.protocol_manager.contract_id().to_string(),
        "fpt_staking": contracts.fpt_staking.contract_id().to_string(),
        "fpt_token": contracts.fpt_token.contract_id().to_string(),
        "community_issuance": contracts.community_issuance.contract_id().to_string(),
        "coll_surplus_pool": contracts.coll_surplus_pool.contract_id().to_string(),
        "default_pool": contracts.default_pool.contract_id().to_string(),
        "active_pool": contracts.active_pool.contract_id().to_string(),
        "sorted_troves": contracts.sorted_troves.contract_id().to_string(),
        "asset_contracts" : contracts.asset_contracts.iter().map(|asset_contracts| {
            json!({
                "oracle": asset_contracts.oracle.contract_id().to_string(),
                "trove_manager": asset_contracts.trove_manager.contract_id().to_string(),
                "asset": asset_contracts.asset.contract_id().to_string(),
            })
        }).collect::<Vec<serde_json::Value>>()
    });

    let _ = file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes());
}

use super::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, community_issuance::community_issuance_abi,
    default_pool::DefaultPool, fpt_staking::FPTStaking, fpt_token::fpt_token_abi, oracle::Oracle,
    protocol_manager::ProtocolManager, sorted_troves::SortedTroves, stability_pool::StabilityPool,
    token::Token, trove_manager::TroveManagerContract, usdf_token::USDFToken,
};

pub mod deployment {

    use fuels::{prelude::Account, programs::call_response::FuelCallResponse, types::Identity};
    use pbr::ProgressBar;

    use super::*;
    use crate::{
        data_structures::PRECISION,
        interfaces::{
            active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
            coll_surplus_pool::coll_surplus_pool_abi, default_pool::default_pool_abi,
            fpt_staking::fpt_staking_abi, oracle::oracle_abi,
            protocol_manager::protocol_manager_abi, sorted_troves::sorted_troves_abi,
            stability_pool::stability_pool_abi, token::token_abi, trove_manager::trove_manager_abi,
            usdf_token::usdf_token_abi,
        },
        setup::common::{
            deploy_active_pool, deploy_borrow_operations, deploy_coll_surplus_pool,
            deploy_community_issuance, deploy_default_pool, deploy_fpt_staking, deploy_fpt_token,
            deploy_oracle, deploy_protocol_manager, deploy_sorted_troves, deploy_stability_pool,
            deploy_token, deploy_trove_manager_contract, deploy_usdf_token, AssetContracts,
            ProtocolContracts,
        },
    };

    pub async fn deploy_and_initialize_all(
        wallet: WalletUnlocked,
        _max_size: u64,
        deploy_2nd_asset: bool,
        existing_eth_contracts: Option<ExistingAssetContracts>,
        existing_st_eth_contracts: Option<ExistingAssetContracts>,
    ) -> ProtocolContracts<WalletUnlocked> {
        println!("Deploying parent contracts...");
        let mut pb = ProgressBar::new(12);

        let borrow_operations = deploy_borrow_operations(&wallet).await;
        pb.inc();

        let usdf = deploy_usdf_token(&wallet).await;
        pb.inc();

        let fpt_token = deploy_fpt_token(&wallet).await;
        pb.inc();

        let _fpt = deploy_token(&wallet).await;
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

        let fuel_asset_contracts = upload_asset(wallet.clone(), &existing_eth_contracts).await;

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

        let mut pb = ProgressBar::new(7);

        let mut asset_contracts: Vec<AssetContracts<WalletUnlocked>> = vec![];
        wait();

        let _ = community_issuance_abi::initialize(
            &community_issuance,
            stability_pool.contract_id().into(),
            fpt_token.contract_id().into(),
            &Identity::Address(wallet.address().into()),
            false,
        )
        .await;
        pb.inc();

        fpt_token_abi::initialize(
            &fpt_token,
            "FPT Token".to_string(),
            "FPT".to_string(),
            &usdf, // TODO this will be the vesting contract
            &community_issuance,
        )
        .await;
        pb.inc();

        let _ = usdf_token_abi::initialize(
            &usdf,
            "USD Fuel".to_string(),
            "USDF".to_string(),
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
            fpt_token.contract_id().into(),
            usdf.contract_id().into(),
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

        let _ = sorted_troves_abi::initialize(
            &sorted_troves,
            100,
            protocol_manager.contract_id().into(),
            borrow_operations.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        initialize_asset(
            &borrow_operations,
            &fpt_staking,
            &stability_pool,
            &protocol_manager,
            &usdf,
            &coll_surplus_pool,
            wallet.clone(),
            "Fuel".to_string(),
            "FUEL".to_string(),
            &default_pool,
            &active_pool,
            &fuel_asset_contracts.asset,
            &fuel_asset_contracts.trove_manager,
            &sorted_troves,
            &fuel_asset_contracts.oracle,
            existing_eth_contracts,
        )
        .await;

        if deploy_2nd_asset {
            let stfuel_asset_contracts =
                upload_asset(wallet.clone(), &existing_st_eth_contracts).await;

            initialize_asset(
                &borrow_operations,
                &fpt_staking,
                &stability_pool,
                &protocol_manager,
                &usdf,
                &coll_surplus_pool,
                wallet.clone(),
                "stFuel".to_string(),
                "stFUEL".to_string(),
                &default_pool,
                &active_pool,
                &stfuel_asset_contracts.asset,
                &stfuel_asset_contracts.trove_manager,
                &sorted_troves,
                &stfuel_asset_contracts.oracle,
                existing_st_eth_contracts,
            )
            .await;

            asset_contracts.push(stfuel_asset_contracts);
        }
        pb.finish();

        asset_contracts.push(fuel_asset_contracts);

        let contracts = ProtocolContracts {
            borrow_operations,
            usdf,
            stability_pool,
            protocol_manager,
            asset_contracts,
            fpt_staking,
            fpt_token,
            fpt: _fpt,
            community_issuance,
            coll_surplus_pool,
            default_pool,
            sorted_troves,
            active_pool,
        };

        return contracts;
    }

    pub fn print_response<T>(response: &FuelCallResponse<T>)
    where
        T: std::fmt::Debug,
    {
        response.receipts.iter().for_each(|r| match r.ra() {
            Some(r) => println!("{:?}", r),
            _ => (),
        });
    }

    pub fn assert_within_threshold(a: u64, b: u64, comment: &str) {
        let threshold = a / 100000;
        assert!(
            a >= b.saturating_sub(threshold) && a <= b.saturating_add(threshold),
            "{}",
            comment
        );
    }

    pub fn wait() {
        std::thread::sleep(std::time::Duration::from_secs(12));
    }

    pub async fn upload_asset(
        wallet: WalletUnlocked,
        existing_contracts: &Option<ExistingAssetContracts>,
    ) -> AssetContracts<WalletUnlocked> {
        println!("Deploying asset contracts...");
        let mut pb = ProgressBar::new(3);
        let trove_manager = deploy_trove_manager_contract(&wallet).await;
        pb.inc();

        match existing_contracts {
            Some(contracts) => {
                pb.finish();
                return AssetContracts {
                    oracle: Oracle::new(contracts.oracle.into(), wallet.clone()),
                    asset: Token::new(contracts.asset.into(), wallet.clone()),
                    trove_manager,
                };
            }
            None => {
                let oracle = deploy_oracle(&wallet).await;
                pb.inc();
                let asset = deploy_token(&wallet).await;
                pb.inc();

                println!("Deploying asset contracts... Done");
                println!("Oracle: {}", oracle.contract_id());
                println!("Trove Manager: {}", trove_manager.contract_id());
                println!("Asset: {}", asset.contract_id());

                return AssetContracts {
                    oracle,
                    trove_manager,
                    asset,
                };
            }
        }
    }

    pub async fn initialize_asset<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        fpt_staking: &FPTStaking<T>,
        stability_pool: &StabilityPool<T>,
        protocol_manager: &ProtocolManager<T>,
        usdf: &USDFToken<T>,
        coll_surplus_pool: &CollSurplusPool<T>,
        wallet: WalletUnlocked,
        name: String,
        symbol: String,
        default_pool: &DefaultPool<T>,
        active_pool: &ActivePool<T>,
        asset: &Token<T>,
        trove_manager: &TroveManagerContract<T>,
        sorted_troves: &SortedTroves<T>,
        oracle: &Oracle<T>,
        existing_contracts: Option<ExistingAssetContracts>,
    ) -> () {
        println!("Initializing asset contracts...");
        let mut pb = ProgressBar::new(7);

        match existing_contracts {
            Some(_) => {}
            None => {
                let _ = token_abi::initialize(
                    &asset,
                    1_000_000_000,
                    &Identity::Address(wallet.address().into()),
                    name.to_string(),
                    symbol.to_string(),
                )
                .await;
                wait();
                pb.inc();

                let _ = oracle_abi::set_price(&oracle, 1000 * PRECISION).await;
                wait();
                pb.inc();
            }
        }

        let _ = trove_manager_abi::initialize(
            &trove_manager,
            borrow_operations.contract_id().into(),
            sorted_troves.contract_id().into(),
            oracle.contract_id().into(),
            stability_pool.contract_id().into(),
            default_pool.contract_id().into(),
            active_pool.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            usdf.contract_id().into(),
            asset.contract_id().into(),
            protocol_manager.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = protocol_manager_abi::register_asset(
            &protocol_manager,
            asset.contract_id().into(),
            trove_manager.contract_id().into(),
            oracle.contract_id().into(),
            borrow_operations,
            stability_pool,
            usdf,
            fpt_staking,
            coll_surplus_pool,
            default_pool,
            active_pool,
            sorted_troves,
        )
        .await;
        wait();
        pb.inc();
    }
}
