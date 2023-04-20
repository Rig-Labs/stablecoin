use fuels::prelude::{abigen, ContractId};

use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

abigen!(Contract(
    name = "SortedTroves",
    abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
));

pub mod sorted_troves_abi {
    use fuels::prelude::{Account, LogDecoder, TxParameters};

    use crate::setup::common::wait;

    use super::*;

    pub async fn initialize<T: Account>(
        sorted_troves: &SortedTroves<T>,
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

    pub async fn insert<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
        icr: u64,
        prev_id: Identity,
        next_id: Identity,
    ) -> FuelCallResponse<()> {
        sorted_troves
            .methods()
            .insert(id, icr, prev_id, next_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_first<T: Account>(
        sorted_troves: &SortedTroves<T>,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_first().call().await.unwrap()
    }

    pub async fn get_last<T: Account>(
        sorted_troves: &SortedTroves<T>,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_last().call().await.unwrap()
    }

    pub async fn get_size<T: Account>(sorted_troves: &SortedTroves<T>) -> FuelCallResponse<u64> {
        sorted_troves.methods().get_size().call().await.unwrap()
    }

    pub async fn get_next<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_next(id).call().await.unwrap()
    }

    pub async fn get_prev<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_prev(id).call().await.unwrap()
    }
}
