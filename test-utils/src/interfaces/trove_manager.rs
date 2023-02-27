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

    pub async fn increase_trove_coll(
        trove_manager: &TroveManagerContract,
        id: Identity,
        amount: u64,
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .increase_trove_coll(id, amount)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_trove_debt(
        trove_manager: &TroveManagerContract,
        id: Identity,
        amount: u64,
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .increase_trove_debt(id, amount)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_trove_status(
        trove_manager: &TroveManagerContract,
        id: Identity,
        status: Status,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::new(Some(1), Some(100_000_000), Some(0));

        trove_manager
            .methods()
            .set_trove_status(id, status)
            .tx_params(tx_params)
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

    pub async fn get_trove_coll(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_trove_coll(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_trove_debt(
        trove_manager: &TroveManagerContract,
        id: Identity,
    ) -> FuelCallResponse<u64> {
        trove_manager
            .methods()
            .get_trove_debt(id)
            .call()
            .await
            .unwrap()
    }
}

pub mod trove_manager_utils {
    use crate::interfaces::sorted_troves::sorted_troves_abi;

    use super::*;

    pub async fn set_coll_and_debt_insert(
        trove_manager: &TroveManagerContract,
        sorted_troves: &SortedTroves,
        id: Identity,
        coll: u64,
        debt: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        trove_manager_abi::increase_trove_coll(trove_manager, id.clone(), coll).await;
        trove_manager_abi::increase_trove_debt(trove_manager, id.clone(), debt).await;
        trove_manager_abi::set_trove_status(trove_manager, id.clone(), Status::Active).await;
        sorted_troves_abi::insert(sorted_troves, id, coll, prev_id, next_id).await;
    }
}
