use fuels::prelude::{abigen, ContractId};

use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

abigen!(Contract(
    name = "SortedTroves",
    abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
));

pub mod sorted_troves_abi {
    use fuels::prelude::{LogDecoder, TxParameters};

    use crate::setup::common::wait;

    use super::*;

    pub async fn initialize(
        sorted_troves: &SortedTroves,
        max_size: u64,
        borrow_opperations: ContractId,
        trove_manager: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = sorted_troves
            .methods()
            .set_params(max_size, trove_manager, borrow_opperations)
            .tx_params(tx_params)
            .call()
            .await;

        // TODO: remove this workaround
        match res {
            Ok(res) => res,
            Err(_) => {
                wait();
                return FuelCallResponse::new((), vec![], LogDecoder::default());
            }
        }
    }

    pub async fn insert(
        sorted_troves: &SortedTroves,
        id: Identity,
        icr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: ContractId,
    ) -> FuelCallResponse<()> {
        sorted_troves
            .methods()
            .insert(id, icr, prev_id, next_id, asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_first(
        sorted_troves: &SortedTroves,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_first(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_last(
        sorted_troves: &SortedTroves,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_last(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_size(sorted_troves: &SortedTroves) -> FuelCallResponse<u64> {
        sorted_troves.methods().get_size().call().await.unwrap()
    }

    pub async fn get_next(
        sorted_troves: &SortedTroves,
        id: Identity,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_next(id, asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_prev(
        sorted_troves: &SortedTroves,
        id: Identity,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_prev(id, asset)
            .call()
            .await
            .unwrap()
    }
}
