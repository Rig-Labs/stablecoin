use fuels::prelude::*;
use fuels::programs::call_response::FuelCallResponse;
use fuels::signers::fuel_crypto::rand::{self, Rng};
use fuels::types::Identity;

use crate::utils::setup::{SortedTroves, TroveManagerContract};

pub mod trove_manager_abi_calls {

    use super::*;

    pub async fn set_nominal_icr_and_insert(
        trove_manager: &TroveManagerContract,
        sorted_troves: &SortedTroves,
        new_id: Identity,
        new_icr: u64,
        prev_id: Identity,
        next_id: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .set_nominal_icr_and_insert(new_id, new_icr, prev_id, next_id)
            .set_contracts(&[sorted_troves])
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }
}

pub async fn deploy_trove_manager_contract(wallet: &WalletUnlocked) -> TroveManagerContract {
    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();

    let id = Contract::deploy(
        &get_path("../trove-manager-contract/out/debug/trove-manager-contract.bin".to_string()),
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(get_path(
            "../trove-manager-contract/out/debug/trove-manager-contract-storage_slots.json"
                .to_string(),
        ))),
    )
    .await
    .unwrap();

    TroveManagerContract::new(id, wallet.clone())
}

fn get_path(sub_path: String) -> String {
    let mut path = std::env::current_dir().unwrap();
    path.push(sub_path);
    path.to_str().unwrap().to_string()
}
