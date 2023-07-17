use super::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations,
    coll_surplus_pool::CollSurplusPool, community_issuance::CommunityIssuance,
    default_pool::DefaultPool, fpt_staking::FPTStaking, fpt_token::FPTToken, oracle::Oracle,
    protocol_manager::ProtocolManager, sorted_troves::SortedTroves, stability_pool::StabilityPool,
    token::Token, trove_manager::TroveManagerContract, usdf_token::USDFToken,
    vesting::VestingContract,
};

use fuels::prelude::{Contract, TxParameters, WalletUnlocked};

pub mod common {

    use std::env;

    use super::*;
    use crate::{
        data_structures::PRECISION,
        interfaces::{
            active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
            coll_surplus_pool::coll_surplus_pool_abi, community_issuance::community_issuance_abi,
            default_pool::default_pool_abi, fpt_staking::fpt_staking_abi, fpt_token::fpt_token_abi,
            oracle::oracle_abi, protocol_manager::protocol_manager_abi,
            sorted_troves::sorted_troves_abi, stability_pool::stability_pool_abi, token::token_abi,
            trove_manager::trove_manager_abi, usdf_token::usdf_token_abi,
        },
        paths::*,
    };
    use fuels::{
        accounts::fuel_crypto::rand::{self, Rng},
        prelude::{
            launch_custom_provider_and_get_wallets, Account, LoadConfiguration, WalletsConfig,
        },
        programs::call_response::FuelCallResponse,
        types::Identity,
    };
    use pbr::ProgressBar;

    pub struct ProtocolContracts<T: Account> {
        pub borrow_operations: BorrowOperations<T>,
        pub usdf: USDFToken<T>,
        pub stability_pool: StabilityPool<T>,
        pub protocol_manager: ProtocolManager<T>,
        pub asset_contracts: Vec<AssetContracts<T>>,
        pub fpt_staking: FPTStaking<T>,
        pub coll_surplus_pool: CollSurplusPool<T>,
        pub sorted_troves: SortedTroves<T>,
        pub default_pool: DefaultPool<T>,
        pub active_pool: ActivePool<T>,
        pub fpt_token: FPTToken<T>,
        pub fpt: Token<T>,
        pub community_issuance: CommunityIssuance<T>,
    }

    pub struct AssetContracts<T: Account> {
        pub asset: Token<T>,
        pub oracle: Oracle<T>,
        pub trove_manager: TroveManagerContract<T>,
    }

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
        let mut pb = ProgressBar::new(8);

        let borrow_operations = deploy_borrow_operations(&wallet).await;
        pb.inc();

        let usdf = deploy_usdf_token(&wallet).await;
        pb.inc();

        let stability_pool = deploy_stability_pool(&wallet).await;
        pb.inc();

        let fpt_staking = deploy_fpt_staking(&wallet).await;
        pb.inc();

        let community_issuance = deploy_community_issuance(&wallet).await;
        pb.inc();

        let fpt_token = deploy_fpt_token(&wallet).await;
        pb.inc();

        let fpt = deploy_token(&wallet).await;
        pb.inc();

        let community_issuance = deploy_community_issuance(&wallet).await;
        pb.inc();

        let fpt_token = deploy_fpt_token(&wallet).await;
        pb.inc();

        let fpt = deploy_token(&wallet).await;
        pb.inc();

        let protocol_manager = deploy_protocol_manager(&wallet).await;
        pb.inc();

        let coll_surplus_pool = deploy_coll_surplus_pool(&wallet).await;

        pb.inc();
        let default_pool = deploy_default_pool(&wallet).await;

        pb.inc();
        let active_pool = deploy_active_pool(&wallet).await;

        pb.inc();
        let sorted_troves = deploy_sorted_troves(&wallet).await;

        pb.finish_println("Parent Contracts deployed");

        if is_testnet {
            println!("Borrow operations: {}", borrow_operations.contract_id());
            println!("Usdf: {}", usdf.contract_id());
            println!("Stability Pool: {}", stability_pool.contract_id());
            println!("Protocol Manager: {}", protocol_manager.contract_id());
            println!("FPT Staking: {}", fpt_staking.contract_id());
            println!("FPT Token: {}", fpt_token.contract_id());
            println!("Mock FPT Token: {}", fpt_token.contract_id());
            println!("Community Issuance: {}", community_issuance.contract_id());
            println!("Coll Surplus Pool: {}", coll_surplus_pool.contract_id());
            println!("Default Pool: {}", default_pool.contract_id());
        }

        let mut pb = ProgressBar::new(8);

        let mut asset_contracts: Vec<AssetContracts<WalletUnlocked>> = vec![];

        community_issuance_abi::initialize(
            &community_issuance,
            stability_pool.contract_id().into(),
            fpt_token.contract_id().into(),
            &Identity::Address(wallet.address().into()),
            true,
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

        // mock token for testing staking
        token_abi::initialize(
            &fpt,
            1_000_000_000,
            &Identity::Address(wallet.address().into()),
            "FPT Token".to_string(),
            "FPT".to_string(),
        )
        .await;
        pb.inc();

        usdf_token_abi::initialize(
            &usdf,
            "USD Fuel".to_string(),
            "USDF".to_string(),
            protocol_manager.contract_id().into(),
            Identity::ContractId(stability_pool.contract_id().into()),
            Identity::ContractId(borrow_operations.contract_id().into()),
        )
        .await;
        pb.inc();

        borrow_operations_abi::initialize(
            &borrow_operations,
            usdf.contract_id().into(),
            fpt_staking.contract_id().into(),
            stability_pool.contract_id().into(),
            protocol_manager.contract_id().into(),
            coll_surplus_pool.contract_id().into(),
            active_pool.contract_id().into(),
            sorted_troves.contract_id().into(),
        )
        .await;
        pb.inc();

        stability_pool_abi::initialize(
            &stability_pool,
            borrow_operations.contract_id().into(),
            usdf.contract_id().into(),
            community_issuance.contract_id().into(),
            protocol_manager.contract_id().into(),
            active_pool.contract_id().into(),
        )
        .await
        .unwrap();
        pb.inc();

        fpt_staking_abi::initialize(
            &fpt_staking,
            protocol_manager.contract_id().into(),
            borrow_operations.contract_id().into(),
            fpt.contract_id().into(), // TODO switch this from `fpt` to `fpt_token`, mock token for testing
            usdf.contract_id().into(),
        )
        .await;
        pb.inc();

        protocol_manager_abi::initialize(
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
        pb.inc();

        coll_surplus_pool_abi::initialize(
            &coll_surplus_pool,
            borrow_operations.contract_id().into(),
            Identity::ContractId(protocol_manager.contract_id().into()),
        )
        .await;

        default_pool_abi::initialize(
            &default_pool,
            Identity::ContractId(protocol_manager.contract_id().into()),
            active_pool.contract_id().into(),
        )
        .await;

        active_pool_abi::initialize(
            &active_pool,
            Identity::ContractId(borrow_operations.contract_id().into()),
            Identity::ContractId(stability_pool.contract_id().into()),
            default_pool.contract_id().into(),
            Identity::ContractId(protocol_manager.contract_id().into()),
        )
        .await;

        sorted_troves_abi::initialize(
            &sorted_troves,
            100,
            protocol_manager.contract_id().into(),
            borrow_operations.contract_id().into(),
        )
        .await;

        let fuel_asset_contracts = add_asset(
            &borrow_operations,
            &stability_pool,
            &protocol_manager,
            &usdf,
            &fpt_staking,
            &coll_surplus_pool,
            &default_pool,
            &active_pool,
            &sorted_troves,
            wallet.clone(),
            "Fuel".to_string(),
            "FUEL".to_string(),
            is_testnet,
        )
        .await;

        if deploy_2nd_asset {
            let usdf_asset_contracts = add_asset(
                &borrow_operations,
                &stability_pool,
                &protocol_manager,
                &usdf,
                &fpt_staking,
                &coll_surplus_pool,
                &default_pool,
                &active_pool,
                &sorted_troves,
                wallet,
                "stFuel".to_string(),
                "stFUEL".to_string(),
                is_testnet,
            )
            .await;
            pb.finish();

            asset_contracts.push(usdf_asset_contracts);
        }
        pb.finish();

        asset_contracts.push(fuel_asset_contracts);

        let contracts = ProtocolContracts {
            borrow_operations,
            usdf,
            stability_pool,
            asset_contracts,
            protocol_manager,
            fpt_staking,
            fpt_token,
            fpt,
            coll_surplus_pool,
            default_pool,
            active_pool,
            sorted_troves,
            community_issuance,
        };

        return contracts;
    }

    pub async fn deploy_token(wallet: &WalletUnlocked) -> Token<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(TOKEN_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => return Token::new(id, wallet.clone()),
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(TOKEN_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return Token::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_fpt_token(wallet: &WalletUnlocked) -> FPTToken<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(FPT_TOKEN_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => return FPTToken::new(id, wallet.clone()),
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(FPT_TOKEN_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return FPTToken::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_sorted_troves(wallet: &WalletUnlocked) -> SortedTroves<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(SORTED_TROVES_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => return SortedTroves::new(id, wallet.clone()),
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(SORTED_TROVES_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return SortedTroves::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_trove_manager_contract(
        wallet: &WalletUnlocked,
    ) -> TroveManagerContract<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(TROVE_MANAGER_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => return TroveManagerContract::new(id, wallet.clone()),
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(TROVE_MANAGER_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return TroveManagerContract::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_vesting_contract(
        wallet: &WalletUnlocked,
    ) -> VestingContract<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(VESTING_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => return VestingContract::new(id, wallet.clone()),
            Err(_) => {
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(VESTING_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return VestingContract::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_oracle(wallet: &WalletUnlocked) -> Oracle<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(ORACLE_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return Oracle::new(id, wallet.clone());
            }
            Err(_) => {
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(ORACLE_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return Oracle::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_protocol_manager(
        wallet: &WalletUnlocked,
    ) -> ProtocolManager<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(PROTCOL_MANAGER_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await
        .unwrap();

        ProtocolManager::new(id, wallet.clone())
    }

    pub async fn deploy_borrow_operations(
        wallet: &WalletUnlocked,
    ) -> BorrowOperations<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(BORROW_OPERATIONS_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return BorrowOperations::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(BORROW_OPERATIONS_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return BorrowOperations::new(id, wallet.clone());
            }
        }
    }

    pub fn get_absolute_path_from_relative(relative_path: &str) -> String {
        let mut path = env::current_dir().unwrap();
        println!("Current directory: {:?}", path);
        let fluid_protocol_index = path
            .to_str()
            .unwrap()
            .find("fluid-protocol/")
            .unwrap_or_else(|| path.to_str().unwrap().len());

        path.push("fluid-protocol/");
        // length of fluid procol
        let len = "fluid-protocol/".len();

        let mut result =
            path.to_str().unwrap().to_string()[..fluid_protocol_index + len].to_string();

        result.push_str(relative_path);

        result
    }

    pub async fn add_asset(
        borrow_operations: &BorrowOperations<WalletUnlocked>,
        stability_pool: &StabilityPool<WalletUnlocked>,
        protocol_manager: &ProtocolManager<WalletUnlocked>,
        usdf: &USDFToken<WalletUnlocked>,
        fpt_staking: &FPTStaking<WalletUnlocked>,
        coll_surplus_pool: &CollSurplusPool<WalletUnlocked>,
        default_pool: &DefaultPool<WalletUnlocked>,
        active_pool: &ActivePool<WalletUnlocked>,
        sorted_troves: &SortedTroves<WalletUnlocked>,
        wallet: WalletUnlocked,
        name: String,
        symbol: String,
        is_testnet: bool,
    ) -> AssetContracts<WalletUnlocked> {
        let oracle = deploy_oracle(&wallet).await;
        let trove_manager = deploy_trove_manager_contract(&wallet).await;
        let asset = deploy_token(&wallet).await;

        if is_testnet {
            println!("Deployed asset: {}", asset.contract_id());
            println!("Deployed trove manager: {}", trove_manager.contract_id());
            println!("Deployed oracle: {}", oracle.contract_id());
            println!("Deployed fpt staking: {}", fpt_staking.contract_id());
        }

        token_abi::initialize(
            &asset,
            1_000_000_000,
            &Identity::Address(wallet.address().into()),
            name.to_string(),
            symbol.to_string(),
        )
        .await;

        trove_manager_abi::initialize(
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

        oracle_abi::set_price(&oracle, 1 * PRECISION).await;

        protocol_manager_abi::register_asset(
            &protocol_manager,
            asset.contract_id().into(),
            trove_manager.contract_id().into(),
            oracle.contract_id().into(),
            borrow_operations,
            stability_pool,
            usdf,
            fpt_staking,
            &coll_surplus_pool,
            default_pool,
            active_pool,
            &sorted_troves,
        )
        .await;

        return AssetContracts {
            oracle,
            trove_manager,
            asset,
        };
    }

    pub async fn deploy_active_pool(wallet: &WalletUnlocked) -> ActivePool<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(ACTIVE_POOL_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return ActivePool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(ACTIVE_POOL_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return ActivePool::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_stability_pool(wallet: &WalletUnlocked) -> StabilityPool<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(STABILITY_POOL_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return StabilityPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(STABILITY_POOL_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return StabilityPool::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_default_pool(wallet: &WalletUnlocked) -> DefaultPool<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(DEFAULT_POOL_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return DefaultPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(DEFAULT_POOL_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return DefaultPool::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_coll_surplus_pool(
        wallet: &WalletUnlocked,
    ) -> CollSurplusPool<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(COLL_SURPLUS_POOL_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return CollSurplusPool::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(COLL_SURPLUS_POOL_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return CollSurplusPool::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_community_issuance(
        wallet: &WalletUnlocked,
    ) -> CommunityIssuance<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(COMMUNITY_ISSUANCE_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return CommunityIssuance::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(COMMUNITY_ISSUANCE_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return CommunityIssuance::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_fpt_staking(wallet: &WalletUnlocked) -> FPTStaking<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(FPT_STAKING_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return FPTStaking::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(FPT_STAKING_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return FPTStaking::new(id, wallet.clone());
            }
        }
    }

    pub async fn deploy_usdf_token(wallet: &WalletUnlocked) -> USDFToken<WalletUnlocked> {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();
        let tx_parms = TxParameters::default().set_gas_price(1);

        let id = Contract::load_from(
            &get_absolute_path_from_relative(USDF_TOKEN_CONTRACT_BINARY_PATH),
            LoadConfiguration::default().set_salt(salt),
        )
        .unwrap()
        .deploy(&wallet.clone(), tx_parms)
        .await;

        match id {
            Ok(id) => {
                return USDFToken::new(id, wallet.clone());
            }
            Err(_) => {
                wait();
                let id = Contract::load_from(
                    &get_absolute_path_from_relative(USDF_TOKEN_CONTRACT_BINARY_PATH),
                    LoadConfiguration::default().set_salt(salt),
                )
                .unwrap()
                .deploy(&wallet.clone(), tx_parms)
                .await
                .unwrap();

                return USDFToken::new(id, wallet.clone());
            }
        }
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
}
