use dotenv::dotenv;
use fuels::{
    prelude::{Address, Bech32Address, Bech32ContractId, Provider, WalletUnlocked},
    types::ContractId,
};

use crate::{
    interfaces::trove_manager::trove_manager_abi,
    setup::common::{deploy_and_initialize_all, ProtocolContracts},
};

const RPC: &str = "beta-3.fuel.network";
// const RPC: &str = "http://localhost:4000";

// #[tokio::test]
pub async fn deploy() {
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

    let contracts: ProtocolContracts<WalletUnlocked> =
        deployment::deploy_and_initialize_all(wallet, 100, true, false).await;

    println!(
        "Borrow operations: {}",
        contracts.borrow_operations.contract_id()
    );
    println!(
        "Oracle: {}",
        contracts.asset_contracts[0].oracle.contract_id()
    );
    println!(
        "Sorted Troves: {}",
        contracts.asset_contracts[0].sorted_troves.contract_id()
    );
    println!(
        "Trove Manager: {}",
        contracts.asset_contracts[0].trove_manager.contract_id()
    );
    println!("Fuel: {}", contracts.asset_contracts[0].asset.contract_id());
    println!("Usdf: {}", contracts.usdf.contract_id());
    println!(
        "Active Pool: {}",
        contracts.asset_contracts[0].active_pool.contract_id()
    );
    println!("Stability Pool: {}", contracts.stability_pool.contract_id());

    println!(
        "Default Pool: {}",
        contracts.asset_contracts[0].default_pool.contract_id()
    );
    println!(
        "Collateral Surplus Pool: {}",
        contracts.asset_contracts[0].coll_surplus_pool.contract_id()
    );
}

use super::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, default_pool::DefaultPool, oracle::Oracle,
    protocol_manager::ProtocolManager, sorted_troves::SortedTroves, stability_pool::StabilityPool,
    token::Token, trove_manager::TroveManagerContract, usdf_token::USDFToken,
};

pub mod deployment {

    use fuels::{
        prelude::{launch_custom_provider_and_get_wallets, Account, WalletsConfig},
        programs::call_response::FuelCallResponse,
        types::Identity,
    };
    use pbr::ProgressBar;

    use super::*;
    use crate::{
        interfaces::{
            active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
            coll_surplus_pool::coll_surplus_pool_abi, default_pool::default_pool_abi,
            oracle::oracle_abi, protocol_manager::protocol_manager_abi,
            sorted_troves::sorted_troves_abi, stability_pool::stability_pool_abi, token::token_abi,
            trove_manager::trove_manager_abi, usdf_token::usdf_token_abi,
        },
        setup::common::{
            deploy_active_pool, deploy_borrow_operations, deploy_coll_surplus_pool,
            deploy_default_pool, deploy_oracle, deploy_protocol_manager, deploy_sorted_troves,
            deploy_stability_pool, deploy_token, deploy_trove_manager_contract, deploy_usdf_token,
            AssetContracts, ProtocolContracts,
        },
    };

    pub async fn setup_protocol(
        max_size: u64,
        num_wallets: u64,
        deploy_2nd_asset: bool,
    ) -> (
        ProtocolContracts<WalletUnlocked>,
        WalletUnlocked,
        Vec<WalletUnlocked>,
    ) {
        // Launch a local network and deploy the contract
        let mut wallets = launch_custom_provider_and_get_wallets(
            WalletsConfig::new(
                Some(num_wallets),   /* Single wallet */
                Some(1),             /* Single coin (UTXO) */
                Some(1_000_000_000), /* Amount per coin */
            ),
            None,
            None,
        )
        .await;
        let wallet = wallets.pop().unwrap();

        let contracts =
            deploy_and_initialize_all(wallet.clone(), max_size, false, deploy_2nd_asset).await;

        (contracts, wallet, wallets)
    }

    pub async fn deploy_and_initialize_all(
        wallet: WalletUnlocked,
        _max_size: u64,
        is_testnet: bool,
        deploy_2nd_asset: bool,
    ) -> ProtocolContracts<WalletUnlocked> {
        println!("Deploying parent contracts...");
        let mut pb = ProgressBar::new(4);

        let borrow_operations = deploy_borrow_operations(&wallet).await;
        pb.inc();

        let usdf = deploy_usdf_token(&wallet).await;
        pb.inc();

        let stability_pool = deploy_stability_pool(&wallet).await;
        pb.inc();

        let protocol_manager = deploy_protocol_manager(&wallet).await;
        pb.inc();

        let fuel_asset_contracts = upload_asset(wallet.clone()).await;

        println!("Borrow operations: {}", borrow_operations.contract_id());
        println!("Usdf: {}", usdf.contract_id());
        println!("Stability Pool: {}", stability_pool.contract_id());
        println!("Initializing contracts...");

        let mut pb = ProgressBar::new(4);

        let asset_contracts: Vec<AssetContracts<WalletUnlocked>> = vec![];
        wait();

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
        wait();

        let _ = borrow_operations_abi::initialize(
            &borrow_operations,
            usdf.contract_id().into(),
            usdf.contract_id().into(),
            stability_pool.contract_id().into(),
            protocol_manager.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = stability_pool_abi::initialize(
            &stability_pool,
            borrow_operations.contract_id().into(),
            usdf.contract_id().into(),
            usdf.contract_id().into(),
            protocol_manager.contract_id().into(),
        )
        .await
        .unwrap();
        wait();
        pb.inc();

        let _ = protocol_manager_abi::initialize(
            &protocol_manager,
            borrow_operations.contract_id().into(),
            stability_pool.contract_id().into(),
            usdf.contract_id().into(),
            Identity::Address(wallet.address().into()),
        )
        .await;
        wait();
        pb.inc();

        initialize_asset(
            &borrow_operations,
            &stability_pool,
            &protocol_manager,
            &usdf,
            wallet,
            "Fuel".to_string(),
            "FUEL".to_string(),
            fuel_asset_contracts.default_pool,
            fuel_asset_contracts.active_pool,
            fuel_asset_contracts.asset,
            fuel_asset_contracts.coll_surplus_pool,
            fuel_asset_contracts.trove_manager,
            fuel_asset_contracts.sorted_troves,
            fuel_asset_contracts.oracle,
        )
        .await;

        // if deploy_2nd_asset {
        //     let usdf_asset_contracts = add_asset(
        //         &borrow_operations,
        //         &stability_pool,
        //         &protocol_manager,
        //         &usdf,
        //         wallet,
        //         "stFuel".to_string(),
        //         "stFUEL".to_string(),
        //         is_testnet,
        //     )
        //     .await;
        //     pb.finish();

        //     asset_contracts.push(usdf_asset_contracts);
        // }
        pb.finish();

        // asset_contracts.push(fuel_asset_contracts);

        let contracts = ProtocolContracts {
            borrow_operations,
            usdf,
            stability_pool,
            asset_contracts,
            protocol_manager,
        };

        return contracts;
    }

    pub fn print_response<T>(response: FuelCallResponse<T>)
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

    pub async fn upload_asset(wallet: WalletUnlocked) -> AssetContracts<WalletUnlocked> {
        println!("Deploying asset contracts...");
        let mut pb = ProgressBar::new(7);
        let oracle = deploy_oracle(&wallet).await;
        pb.inc();
        let sorted_troves = deploy_sorted_troves(&wallet).await;
        pb.inc();
        let trove_manager = deploy_trove_manager_contract(&wallet).await;
        pb.inc();
        let asset = deploy_token(&wallet).await;
        pb.inc();
        let active_pool = deploy_active_pool(&wallet).await;
        pb.inc();
        let default_pool = deploy_default_pool(&wallet).await;
        pb.inc();
        let coll_surplus_pool = deploy_coll_surplus_pool(&wallet).await;
        pb.inc();
        println!("Deploying asset contracts... Done");
        println!("Oracle: {}", oracle.contract_id());
        println!("Sorted Troves: {}", sorted_troves.contract_id());
        println!("Trove Manager: {}", trove_manager.contract_id());
        println!("Asset: {}", asset.contract_id());
        println!("Active Pool: {}", active_pool.contract_id());
        println!("Default Pool: {}", default_pool.contract_id());
        println!("Coll Surplus Pool: {}", coll_surplus_pool.contract_id());

        return AssetContracts {
            oracle,
            sorted_troves,
            trove_manager,
            asset,
            active_pool,
            default_pool,
            coll_surplus_pool,
        };
    }

    pub async fn initialize_asset<T: Account>(
        borrow_operations: &BorrowOperations<T>,
        stability_pool: &StabilityPool<T>,
        protocol_manager: &ProtocolManager<T>,
        usdf: &USDFToken<T>,
        wallet: WalletUnlocked,
        name: String,
        symbol: String,
        default_pool: DefaultPool<T>,
        active_pool: ActivePool<T>,
        asset: Token<T>,
        coll_surplus_pool: CollSurplusPool<T>,
        trove_manager: TroveManagerContract<T>,
        sorted_troves: SortedTroves<T>,
        oracle: Oracle<T>,
    ) -> () {
        println!("Initializing asset contracts...");
        let mut pb = ProgressBar::new(7);
        let _ = default_pool_abi::initialize(
            &default_pool,
            Identity::ContractId(trove_manager.contract_id().into()),
            active_pool.contract_id().into(),
            asset.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = active_pool_abi::initialize(
            &active_pool,
            Identity::ContractId(borrow_operations.contract_id().into()),
            Identity::ContractId(trove_manager.contract_id().into()),
            Identity::ContractId(stability_pool.contract_id().into()),
            asset.contract_id().into(),
            default_pool.contract_id().into(),
            protocol_manager.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

        let _ = coll_surplus_pool_abi::initialize(
            &coll_surplus_pool,
            Identity::ContractId(trove_manager.contract_id().into()),
            active_pool.contract_id().into(),
            borrow_operations.contract_id().into(),
            asset.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

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

        let _ = sorted_troves_abi::initialize(
            &sorted_troves,
            100,
            borrow_operations.contract_id().into(),
            trove_manager.contract_id().into(),
        )
        .await;
        wait();
        pb.inc();

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

        let _ = oracle_abi::set_price(&oracle, 1_000_000).await;
        wait();
        pb.inc();

        let _ = protocol_manager_abi::register_asset(
            &protocol_manager,
            asset.contract_id().into(),
            active_pool.contract_id().into(),
            trove_manager.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            oracle.contract_id().into(),
            sorted_troves.contract_id().into(),
            borrow_operations,
            stability_pool,
            usdf,
        )
        .await;
        wait();
        pb.inc();
    }
}
