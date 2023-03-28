use super::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, default_pool::DefaultPool, oracle::Oracle,
    sorted_troves::SortedTroves, stability_pool::StabilityPool, token::Token,
    trove_manager::TroveManagerContract, usdf_token::USDFToken, vesting::VestingContract,
};

use fuels::prelude::{Contract, StorageConfiguration, TxParameters, WalletUnlocked};

pub mod common {

    use fuels::{
        prelude::{launch_custom_provider_and_get_wallets, DeployConfiguration, WalletsConfig},
        programs::call_response::FuelCallResponse,
        signers::fuel_crypto::rand::{self, Rng},
        types::Identity,
    };
    use pbr::ProgressBar;

    use super::*;
    use crate::{
        interfaces::{
            active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
            coll_surplus_pool::coll_surplus_pool_abi, default_pool::default_pool_abi,
            oracle::oracle_abi, sorted_troves::sorted_troves_abi,
            stability_pool::stability_pool_abi, token::token_abi, trove_manager::trove_manager_abi,
            usdf_token::usdf_token_abi,
        },
        paths::*,
        utils::resolve_relative_path,
    };

    pub struct ProtocolContracts {
        pub borrow_operations: BorrowOperations,
        pub trove_manager: TroveManagerContract,
        pub oracle: Oracle,
        pub sorted_troves: SortedTroves,
        pub fuel: Token,
        pub usdf: USDFToken,
        pub active_pool: ActivePool,
        pub stability_pool: StabilityPool,
        pub default_pool: DefaultPool,
        pub coll_surplus_pool: CollSurplusPool,
    }

    pub async fn setup_protocol(
        max_size: u64,
        num_wallets: u64,
    ) -> (ProtocolContracts, WalletUnlocked, Vec<WalletUnlocked>) {
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

        let contracts = deploy_and_initialize_all(wallet.clone(), max_size, false).await;

        (contracts, wallet, wallets)
    }

    pub async fn deploy_and_initialize_all(
        wallet: WalletUnlocked,
        max_size: u64,
        is_testnet: bool,
    ) -> ProtocolContracts {
        println!("Deploying contracts...");
        let mut pb = ProgressBar::new(10);

        let borrow_operations = deploy_borrow_operations(&wallet).await;
        pb.inc();

        let oracle_instance = deploy_oracle(&wallet).await;
        pb.inc();

        let sorted_troves = deploy_sorted_troves(&wallet).await;
        pb.inc();

        let trove_manager = deploy_trove_manager_contract(&wallet).await;
        pb.inc();

        let fuel = deploy_token(&wallet).await;
        pb.inc();

        let usdf = deploy_usdf_token(&wallet).await;
        pb.inc();

        let active_pool = deploy_active_pool(&wallet).await;
        pb.inc();

        let stability_pool = deploy_stability_pool(&wallet).await;
        pb.inc();

        let default_pool = deploy_default_pool(&wallet).await;
        pb.inc();

        let coll_surplus_pool = deploy_coll_surplus_pool(&wallet).await;
        pb.finish();

        if is_testnet {
            println!("Borrow operations: {}", borrow_operations.contract_id());
            println!("Oracle: {}", oracle_instance.contract_id());
            println!("Sorted Troves: {}", sorted_troves.contract_id());
            println!("Trove Manager: {}", trove_manager.contract_id());
            println!("Fuel: {}", fuel.contract_id());
            println!("Usdf: {}", usdf.contract_id());
            println!("Active Pool: {}", active_pool.contract_id());
            println!("Stability Pool: {}", stability_pool.contract_id());
            println!("Default Pool: {}", default_pool.contract_id());
            println!(
                "Collateral Surplus Pool: {}",
                coll_surplus_pool.contract_id()
            );
            println!("Initializing contracts...");
        }

        let mut pb = ProgressBar::new(10);
        default_pool_abi::initialize(
            &default_pool,
            Identity::ContractId(trove_manager.contract_id().into()),
            active_pool.contract_id().into(),
            fuel.contract_id().into(),
        )
        .await;
        pb.inc();

        active_pool_abi::initialize(
            &active_pool,
            Identity::ContractId(borrow_operations.contract_id().into()),
            Identity::ContractId(trove_manager.contract_id().into()),
            Identity::ContractId(stability_pool.contract_id().into()),
            fuel.contract_id().into(),
            default_pool.contract_id().into(),
        )
        .await;
        pb.inc();

        coll_surplus_pool_abi::initialize(
            &coll_surplus_pool,
            Identity::ContractId(trove_manager.contract_id().into()),
            active_pool.contract_id().into(),
            borrow_operations.contract_id().into(),
            fuel.contract_id().into(),
        )
        .await;
        pb.inc();

        token_abi::initialize(
            &fuel,
            1_000_000_000,
            &Identity::Address(wallet.address().into()),
            "Fuel".to_string(),
            "FUEL".to_string(),
        )
        .await;
        pb.inc();

        usdf_token_abi::initialize(
            &usdf,
            "USD Fuel".to_string(),
            "USDF".to_string(),
            Identity::ContractId(trove_manager.contract_id().into()),
            Identity::ContractId(stability_pool.contract_id().into()),
            Identity::ContractId(borrow_operations.contract_id().into()),
        )
        .await;
        pb.inc();

        sorted_troves_abi::initialize(
            &sorted_troves,
            max_size,
            borrow_operations.contract_id().into(),
            trove_manager.contract_id().into(),
        )
        .await;
        pb.inc();

        trove_manager_abi::initialize(
            &trove_manager,
            borrow_operations.contract_id().into(),
            sorted_troves.contract_id().into(),
            oracle_instance.contract_id().into(),
            stability_pool.contract_id().into(),
            default_pool.contract_id().into(),
            active_pool.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            usdf.contract_id().into(),
        )
        .await;
        pb.inc();

        oracle_abi::set_price(&oracle_instance, 1_000_000).await;
        pb.inc();

        borrow_operations_abi::initialize(
            &borrow_operations,
            trove_manager.contract_id().into(),
            sorted_troves.contract_id().into(),
            oracle_instance.contract_id().into(),
            fuel.contract_id().into(),
            usdf.contract_id().into(),
            usdf.contract_id().into(),
            active_pool.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            stability_pool.contract_id().into(),
        )
        .await;
        pb.inc();

        stability_pool_abi::initialize(
            &stability_pool,
            borrow_operations.contract_id().into(),
            trove_manager.contract_id().into(),
            active_pool.contract_id().into(),
            usdf.contract_id().into(),
            sorted_troves.contract_id().into(),
            oracle_instance.contract_id().into(),
            oracle_instance.contract_id().into(),
            fuel.contract_id().into(),
        )
        .await
        .unwrap();
        pb.finish();

        let contracts = ProtocolContracts {
            borrow_operations: borrow_operations,
            trove_manager: trove_manager,
            oracle: oracle_instance,
            sorted_troves,
            fuel,
            usdf,
            active_pool,
            stability_pool,
            default_pool,
            coll_surplus_pool,
        };

        return contracts;
    }

    pub async fn deploy_token(wallet: &WalletUnlocked) -> Token {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config =
            DeployConfiguration::default()
                .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                    resolve_relative_path(TOKEN_CONTRACT_STORAGE_PATH).to_string(),
                ))
                .set_salt(salt)
                .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(TOKEN_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config,
        )
        .await
        .unwrap();

        Token::new(id, wallet.clone())
    }

    pub async fn deploy_sorted_troves(wallet: &WalletUnlocked) -> SortedTroves {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(SORTED_TROVES_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(SORTED_TROVES_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config,
        )
        .await
        .unwrap();

        SortedTroves::new(id, wallet.clone())
    }

    pub async fn deploy_trove_manager_contract(wallet: &WalletUnlocked) -> TroveManagerContract {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(TROVE_MANAGER_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(TROVE_MANAGER_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config,
        )
        .await
        .unwrap();

        TroveManagerContract::new(id, wallet.clone())
    }

    pub async fn deploy_vesting_contract(wallet: &WalletUnlocked) -> VestingContract {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config =
            DeployConfiguration::default()
                .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                    resolve_relative_path(VESTING_CONTRACT_STORAGE_PATH).to_string(),
                ))
                .set_salt(salt)
                .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(VESTING_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config,
        )
        .await
        .unwrap();

        VestingContract::new(id, wallet.clone())
    }

    pub async fn deploy_oracle(wallet: &WalletUnlocked) -> Oracle {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config =
            DeployConfiguration::default()
                .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                    resolve_relative_path(ORACLE_CONTRACT_STORAGE_PATH).to_string(),
                ))
                .set_salt(salt)
                .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(ORACLE_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return Oracle::new(id, wallet.clone());
            }
            Err(_) => {
                let id = Contract::deploy(
                    &resolve_relative_path(ORACLE_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();
                return Oracle::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_borrow_operations(wallet: &WalletUnlocked) -> BorrowOperations {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(BORROW_OPERATIONS_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(BORROW_OPERATIONS_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return BorrowOperations::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(BORROW_OPERATIONS_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return BorrowOperations::new(try_id, wallet.clone());
            }
        }
    }

    pub async fn deploy_active_pool(wallet: &WalletUnlocked) -> ActivePool {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(ACTIVE_POOL_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(ACTIVE_POOL_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return ActivePool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(ACTIVE_POOL_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return ActivePool::new(try_id, wallet.clone());
            }
        }
    }

    pub async fn deploy_stability_pool(wallet: &WalletUnlocked) -> StabilityPool {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(STABILITY_POOL_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(STABILITY_POOL_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return StabilityPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(STABILITY_POOL_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return StabilityPool::new(try_id, wallet.clone());
            }
        }
    }

    pub async fn deploy_default_pool(wallet: &WalletUnlocked) -> DefaultPool {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(DEFAULT_POOL_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(DEFAULT_POOL_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return DefaultPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(DEFAULT_POOL_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return DefaultPool::new(try_id, wallet.clone());
            }
        }
    }

    pub async fn deploy_coll_surplus_pool(wallet: &WalletUnlocked) -> CollSurplusPool {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(COLL_SURPLUS_POOL_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(COLL_SURPLUS_POOL_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return CollSurplusPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(COLL_SURPLUS_POOL_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return CollSurplusPool::new(try_id, wallet.clone());
            }
        }
    }

    pub async fn deploy_usdf_token(wallet: &WalletUnlocked) -> USDFToken {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let deploy_config = DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                resolve_relative_path(USDF_TOKEN_CONTRACT_STORAGE_PATH).to_string(),
            ))
            .set_salt(salt)
            .set_tx_parameters(tx_parms);

        let id = Contract::deploy(
            &resolve_relative_path(USDF_TOKEN_CONTRACT_BINARY_PATH).to_string(),
            &wallet,
            deploy_config.clone(),
        )
        .await;

        match id {
            Ok(id) => {
                return USDFToken::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let try_id = Contract::deploy(
                    &resolve_relative_path(USDF_TOKEN_CONTRACT_BINARY_PATH).to_string(),
                    &wallet,
                    deploy_config,
                )
                .await
                .unwrap();

                return USDFToken::new(try_id, wallet.clone());
            }
        }
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
}
