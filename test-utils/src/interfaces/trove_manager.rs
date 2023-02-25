use fuels::{
    prelude::{abigen, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

use crate::interfaces::sorted_troves::SortedTroves;

abigen!(Contract(
    name = "TroveManagerContract",
    abi = "contracts/trove-manager-contract/out/debug/trove-manager-contract-abi.json"
));

pub mod trove_manager_abi {
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

    pub async fn get_nominal_icr(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_nominal_icr(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn remove(
        trove_manager: &TroveManagerContract,
        sorted_troves: &SortedTroves,
        id: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .remove(id)
            .set_contracts(&[sorted_troves])
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn initialize(
        trove_manager: &TroveManagerContract,
        sorted_troves_id: ContractId,
    ) -> FuelCallResponse<()> {
        trove_manager
            .methods()
            .initialize(sorted_troves_id)
            .call()
            .await
            .unwrap()
    }
}
