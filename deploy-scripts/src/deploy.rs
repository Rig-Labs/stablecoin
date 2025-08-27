use std::{fs::File, io::Write};

use dotenv::dotenv;
use fuels::prelude::*;
use fuels::types::{Address, Bits256};
use serde_json::json;
use test_utils::data_structures::ProtocolContracts;
use test_utils::interfaces::hint_helper::HintHelper;
use test_utils::interfaces::multi_trove_getter::MultiTroveGetter;

use crate::constants::{MAINNET_CONTRACTS_FILE, TESTNET_CONTRACTS_FILE};
use crate::utils::utils::{is_testnet, setup_wallet};

use test_utils::setup::common::{
    deploy_core_contracts, deploy_hint_helper, deploy_multi_trove_getter, initialize_core_contracts,
};

pub mod deployment {

    use std::str::FromStr;

    use super::*;
    pub async fn deploy() {
        //--------------- Deploy ---------------
        dotenv().ok();

        let wallet = setup_wallet().await;
        let network_name = wallet.provider().chain_info().await.unwrap().name;
        let address: Address = wallet.address().into();
        let is_testnet = is_testnet(wallet.clone()).await;
        //--------------- WALLET ---------------
        println!("🔑 Wallet address: 0x{}", address);
        println!("🔑 Is testnet: {}", is_testnet);
        println!("🔑 Network name: {}", network_name);
        //--------------- Deploy ---------------
        let core_contracts =
            deploy_and_initialize_all_core_contracts(wallet.clone(), is_testnet).await;
        let (hint_helper, multi_trove_getter) =
            deploy_frontend_helper_contracts(wallet.clone(), &core_contracts).await;

        //--------------- Write to file ---------------
        write_contracts_to_file(core_contracts, hint_helper, multi_trove_getter, is_testnet);
    }

    pub async fn deploy_and_initialize_all_core_contracts(
        wallet: Wallet,
        _is_testnet: bool,
    ) -> ProtocolContracts<Wallet> {
        let mut core_contracts = deploy_core_contracts(&wallet, false, true).await;
        initialize_core_contracts(&mut core_contracts, &wallet, false, false, true).await;

        return core_contracts;
    }

    pub async fn deploy_frontend_helper_contracts(
        wallet: Wallet,
        core_contracts: &ProtocolContracts<Wallet>,
    ) -> (HintHelper<Wallet>, MultiTroveGetter<Wallet>) {
        let hint_helper = deploy_hint_helper(&wallet).await;
        let multi_trove_getter = deploy_multi_trove_getter(
            &wallet,
            &core_contracts.sorted_troves.contract.contract_id().into(),
        )
        .await;

        return (hint_helper, multi_trove_getter);
    }

    fn write_contracts_to_file(
        contracts: ProtocolContracts<Wallet>,
        hint_helper: HintHelper<Wallet>,
        multi_trove_getter: MultiTroveGetter<Wallet>,
        is_testnet: bool,
    ) {
        let mut file = File::create(match is_testnet {
            true => TESTNET_CONTRACTS_FILE,
            false => MAINNET_CONTRACTS_FILE,
        })
        .unwrap();

        let json = json!({
            "borrow_operations": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.borrow_operations.contract.contract_id().to_string()).unwrap())),
            "borrow_operations_implementation_id": format!("0x{}", contracts.borrow_operations.implementation_id.to_string()),
            "usdm": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.usdm.contract.contract_id().to_string()).unwrap())),
            "usdm_implementation_id": format!("0x{}", contracts.usdm.implementation_id.to_string()),
            "usdm_asset_id": format!("0x{}", contracts.usdm_asset_id.to_string()),
            "stability_pool": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.stability_pool.contract.contract_id().to_string()).unwrap())),
            "stability_pool_implementation_id": format!("0x{}", contracts.stability_pool.implementation_id.to_string()),
            "protocol_manager": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.protocol_manager.contract.contract_id().to_string()).unwrap())),
            "protocol_manager_implementation_id": format!("0x{}", contracts.protocol_manager.implementation_id.to_string()),
            "fpt_staking": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.fpt_staking.contract.contract_id().to_string()).unwrap())),
            "fpt_staking_implementation_id": format!("0x{}", contracts.fpt_staking.implementation_id.to_string()),
            "fpt_token": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.fpt_token.contract.contract_id().to_string()).unwrap())),
            "fpt_token_implementation_id": format!("0x{}", contracts.fpt_token.implementation_id.to_string()),
            "fpt_asset_id": format!("0x{}", contracts.fpt_asset_id.to_string()),
            "community_issuance": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.community_issuance.contract.contract_id().to_string()).unwrap())),
            "community_issuance_implementation_id": format!("0x{}", contracts.community_issuance.implementation_id.to_string()),
            "coll_surplus_pool": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.coll_surplus_pool.contract.contract_id().to_string()).unwrap())),
            "coll_surplus_pool_implementation_id": format!("0x{}", contracts.coll_surplus_pool.implementation_id.to_string()),
            "default_pool": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.default_pool.contract.contract_id().to_string()).unwrap())),
            "default_pool_implementation_id": format!("0x{}", contracts.default_pool.implementation_id.to_string()),
            "active_pool": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.active_pool.contract.contract_id().to_string()).unwrap())),
            "active_pool_implementation_id": format!("0x{}", contracts.active_pool.implementation_id.to_string()),
            "sorted_troves": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.sorted_troves.contract.contract_id().to_string()).unwrap())),
            "sorted_troves_implementation_id": format!("0x{}", contracts.sorted_troves.implementation_id.to_string()),
            "vesting_contract": format!("{:#}", Address::from(Bech32Address::from_str(&contracts.vesting_contract.contract.contract_id().to_string()).unwrap())),
            "vesting_contract_implementation_id": format!("0x{}", contracts.vesting_contract.implementation_id.to_string()),
            "hint_helper": format!("{:#}", Address::from(Bech32Address::from_str(&hint_helper.contract_id().to_string()).unwrap())),
            "multi_trove_getter": format!("{:#}", Address::from(Bech32Address::from_str(&multi_trove_getter.contract_id().to_string()).unwrap())),
            "asset_contracts": contracts.asset_contracts.iter().map(|asset_contracts| {
                json!({
                    "oracle": format!("{:#}", Address::from(Bech32Address::from_str(&asset_contracts.oracle.contract.contract_id().to_string()).unwrap())),
                    "oracle_implementation_id": format!("0x{}", asset_contracts.oracle.implementation_id.to_string()),
                    "trove_manager": format!("{:#}", Address::from(Bech32Address::from_str(&asset_contracts.trove_manager.contract.contract_id().to_string()).unwrap())),
                    "trove_manager_implementation_id": format!("0x{}", asset_contracts.trove_manager.implementation_id.to_string()),
                    "asset_contract": format!("{:#}", Address::from(Bech32Address::from_str(&asset_contracts.asset.contract_id().to_string()).unwrap())),
                    "asset_id": format!("0x{}", asset_contracts.asset_id.to_string()),
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

    pub fn to_hex_str(bits: &Bits256) -> String {
        format!("0x{}", hex::encode(bits.0))
    }
}
