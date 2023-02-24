use super::interfaces::{
    borrow_operations::BorrowOperations, oracle::Oracle, sorted_troves::SortedTroves, token::Token,
    trove_manager::TroveManagerContract, vesting::VestingContract,
};

use fuels::prelude::{Contract, StorageConfiguration, TxParameters, WalletUnlocked};

pub mod common {
    use fuels::{
        prelude::Salt,
        signers::fuel_crypto::rand::{self, Rng},
    };

    use super::*;
    use crate::paths::*;

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
}
