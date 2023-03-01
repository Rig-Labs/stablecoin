use super::interfaces::{
    active_pool::ActivePool, borrow_operations::BorrowOperations, oracle::Oracle,
    sorted_troves::SortedTroves, token::Token, trove_manager::TroveManagerContract,
    vesting::VestingContract,
};

use fuels::prelude::{Contract, StorageConfiguration, TxParameters, WalletUnlocked};

pub mod common {

    use fuels::{
        prelude::{launch_custom_provider_and_get_wallets, Salt, WalletsConfig},
        signers::fuel_crypto::rand::{self, Rng},
        types::Identity,
    };

    use super::*;
    use crate::{
        interfaces::{
            active_pool::active_pool_abi, borrow_operations::borrow_operations_abi,
            oracle::oracle_abi, sorted_troves::sorted_troves_abi, token::token_abi,
            trove_manager::trove_manager_abi,
        },
        paths::*,
    };

    pub async fn setup_protocol(
        max_size: u64,
        num_wallets: u64,
    ) -> (
        BorrowOperations,
        TroveManagerContract,
        Oracle,
        SortedTroves,
        Token, /* Fuel */
        Token, /* USDF */
        ActivePool,
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

        let bo_instance = deploy_borrow_operations(&wallet).await;
        let oracle_instance = deploy_oracle(&wallet).await;
        let sorted_troves = deploy_sorted_troves(&wallet).await;
        let trove_manger = deploy_trove_manager_contract(&wallet).await;
        let fuel = deploy_token(&wallet).await;
        let usdf = deploy_token(&wallet).await;
        let active_pool = deploy_active_pool(&wallet).await;

        // TODO Change stability pool when implemented
        active_pool_abi::initialize(
            &active_pool,
            Identity::ContractId(bo_instance.contract_id().into()),
            Identity::ContractId(trove_manger.contract_id().into()),
            Identity::ContractId(oracle_instance.contract_id().into()),
            fuel.contract_id().into(),
        )
        .await;

        token_abi::initialize(
            &fuel,
            1_000_000_000,
            &Identity::Address(wallet.address().into()),
            "Fuel".to_string(),
            "FUEL".to_string(),
        )
        .await;

        token_abi::initialize(
            &usdf,
            0,
            &Identity::ContractId(bo_instance.contract_id().into()),
            "USD Fuel".to_string(),
            "USDF".to_string(),
        )
        .await;

        sorted_troves_abi::initialize(
            &sorted_troves,
            max_size,
            bo_instance.contract_id().into(),
            trove_manger.contract_id().into(),
        )
        .await;

        trove_manager_abi::initialize(
            &trove_manger,
            bo_instance.contract_id().into(),
            sorted_troves.contract_id().into(),
        )
        .await;

        oracle_abi::set_price(&oracle_instance, 1_000_000).await;

        borrow_operations_abi::initialize(
            &bo_instance,
            trove_manger.contract_id().into(),
            sorted_troves.contract_id().into(),
            oracle_instance.contract_id().into(),
            fuel.contract_id().into(),
            usdf.contract_id().into(),
            usdf.contract_id().into(),
            active_pool.contract_id().into(),
        )
        .await;

        (
            bo_instance,
            trove_manger,
            oracle_instance,
            sorted_troves,
            fuel,
            usdf,
            active_pool,
            wallet,
            wallets,
        )
    }

    pub async fn deploy_token(wallet: &WalletUnlocked) -> Token {
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<[u8; 32]>();

        let id = Contract::deploy_with_parameters(
            &TOKEN_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(TOKEN_CONTRACT_STORAGE_PATH.to_string())),
            Salt::from(salt),
        )
        .await
        .unwrap();

        Token::new(id, wallet.clone())
    }

    pub async fn deploy_sorted_troves(wallet: &WalletUnlocked) -> SortedTroves {
        let id = Contract::deploy(
            &SORTED_TROVES_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                SORTED_TROVES_CONTRACT_STORAGE_PATH.to_string(),
            )),
        )
        .await
        .unwrap();

        SortedTroves::new(id, wallet.clone())
    }

    pub async fn deploy_trove_manager_contract(wallet: &WalletUnlocked) -> TroveManagerContract {
        let id = Contract::deploy(
            &TROVE_MANAGER_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                TROVE_MANAGER_CONTRACT_STORAGE_PATH.to_string(),
            )),
        )
        .await
        .unwrap();

        TroveManagerContract::new(id, wallet.clone())
    }

    pub async fn deploy_vesting_contract(wallet: &WalletUnlocked) -> VestingContract {
        let id = Contract::deploy(
            &VESTING_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                VESTING_CONTRACT_STORAGE_PATH.to_string(),
            )),
        )
        .await
        .unwrap();

        VestingContract::new(id, wallet.clone())
    }

    pub async fn deploy_oracle(wallet: &WalletUnlocked) -> Oracle {
        let id = Contract::deploy(
            &ORACLE_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(ORACLE_CONTRACT_STORAGE_PATH.to_string())),
        )
        .await
        .unwrap();

        Oracle::new(id, wallet.clone())
    }

    pub async fn deploy_borrow_operations(wallet: &WalletUnlocked) -> BorrowOperations {
        let id = Contract::deploy(
            &BORROW_OPERATIONS_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                BORROW_OPERATIONS_CONTRACT_STORAGE_PATH.to_string(),
            )),
        )
        .await
        .unwrap();

        BorrowOperations::new(id, wallet.clone())
    }

    pub async fn deploy_active_pool(wallet: &WalletUnlocked) -> ActivePool {
        let id = Contract::deploy(
            &ACTIVE_POOL_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                ACTIVE_POOL_CONTRACT_STORAGE_PATH.to_string(),
            )),
        )
        .await
        .unwrap();

        ActivePool::new(id, wallet.clone())
    }
}
