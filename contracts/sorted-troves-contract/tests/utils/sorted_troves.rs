use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

use crate::utils::setup::{SortedTroves, TroveManagerContract};

pub mod sorted_troves_abi_calls {

    use super::*;

    pub async fn get_first(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_first().call().await.unwrap()
    }

    pub async fn get_last(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
        sorted_troves.methods().get_last().call().await.unwrap()
    }

    pub async fn get_size(sorted_troves: &SortedTroves) -> FuelCallResponse<u64> {
        sorted_troves.methods().get_size().call().await.unwrap()
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
    use crate::utils::trove_manager::trove_manager_abi_calls;

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

    pub async fn assert_ascending_in_order(
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
    ) {
        let mut count = 0;
        let size = sorted_troves_abi_calls::get_size(sorted_troves).await.value;

        let mut current = sorted_troves_abi_calls::get_first(sorted_troves)
            .await
            .value;

        let mut next = sorted_troves_abi_calls::get_next(sorted_troves, current.clone())
            .await
            .value;

        while next.clone() != Identity::Address([0; 32].into()) {
            let current_icr =
                trove_manager_abi_calls::get_nominal_icr(trove_manager, current.clone())
                    .await
                    .value;

            let next_icr = trove_manager_abi_calls::get_nominal_icr(trove_manager, next.clone())
                .await
                .value;

            assert!(current_icr <= next_icr);

            current = next.clone();
            next = sorted_troves_abi_calls::get_next(&sorted_troves, current.clone())
                .await
                .value
                .clone();

            count += 1;
        }

        assert!(count == size);
    }

    pub async fn assert_descending_in_order(
        sorted_troves: &SortedTroves,
        trove_manager: &TroveManagerContract,
    ) {
        let mut count = 0;
        let size = sorted_troves_abi_calls::get_size(sorted_troves).await.value;

        let mut current = sorted_troves_abi_calls::get_last(&sorted_troves)
            .await
            .value;

        let mut prev = sorted_troves_abi_calls::get_prev(&sorted_troves, current.clone())
            .await
            .value;

        while prev.clone() != Identity::Address([0; 32].into()) {
            let current_icr =
                trove_manager_abi_calls::get_nominal_icr(trove_manager, current.clone())
                    .await
                    .value;

            let prev_icr = trove_manager_abi_calls::get_nominal_icr(trove_manager, prev.clone())
                .await
                .value;

            assert!(current_icr >= prev_icr);

            current = prev.clone();
            prev = sorted_troves_abi_calls::get_prev(&sorted_troves, current.clone())
                .await
                .value
                .clone();
            count += 1;
        }

        assert!(count == size);
    }
}
