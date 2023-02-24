use super::interface::{Token, VestingContract};

use fuels::{
    prelude::{
        Address, AssetId, Bech32Address, Contract, ContractId, Provider, Salt, SettableContract,
        StorageConfiguration, TxParameters, WalletUnlocked,
    },
    tx::Contract as TxContract,
};

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
}
