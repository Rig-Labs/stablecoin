use super::interfaces::{
    oracle::Oracle, token::Token, trove_manager::TroveManagerContract, vesting::VestingContract,
};

use fuels::prelude::{Contract, StorageConfiguration, TxParameters, WalletUnlocked};

pub mod common {
    use super::*;
    use crate::paths::*;

    pub async fn deploy_token(wallet: &WalletUnlocked) -> Token {
        let id = Contract::deploy(
            &TOKEN_CONTRACT_BINARY_PATH.to_string(),
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(TOKEN_CONTRACT_STORAGE_PATH.to_string())),
        )
        .await
        .unwrap();

        Token::new(id, wallet.clone())
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
}
