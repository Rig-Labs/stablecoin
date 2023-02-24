use super::interface::{Token, VestingContract};

use fuels::{
    prelude::{
        Address, AssetId, Bech32Address, Contract, ContractId, Provider, Salt, SettableContract,
        StorageConfiguration, TxParameters, WalletUnlocked,
    },
    tx::Contract as TxContract,
};
