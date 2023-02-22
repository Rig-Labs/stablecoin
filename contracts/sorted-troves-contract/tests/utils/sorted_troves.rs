use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

use crate::utils::setup::SortedTroves;

pub mod sorted_troves_abi_calls {

    use super::*;

    pub async fn get_first(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_first().call().await.unwrap()
    }

    pub async fn get_last(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_last().call().await.unwrap()
    }

    pub async fn get_next(
        sorted_troves: &SortedTroves,
        id: Identity,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_next(id).call().await.unwrap()
    }

    pub async fn get_prev(
        sorted_troves: &SortedTroves,
        id: Identity,
    ) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_prev(id).call().await.unwrap()
    }
}

pub mod sorted_troves_utils {
    use super::*;

    pub async fn assert_neighbors(
        sorted_troves: &SortedTroves,
        current: Identity,
        prev_id: Identity,
        next_id: Identity,
    ) {
        let next = sorted_troves_abi_calls::get_next(&sorted_troves, current.clone()).await;
        assert_eq!(next.value, next_id);

        let prev = sorted_troves_abi_calls::get_prev(&sorted_troves, current.clone()).await;
        assert_eq!(prev.value, prev_id);
    }
}
