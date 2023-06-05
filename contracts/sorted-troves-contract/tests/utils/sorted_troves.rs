use fuels::types::Identity;

use super::setup::MockTroveManagerContract;
use test_utils::interfaces::sorted_troves::sorted_troves_abi;
use test_utils::interfaces::sorted_troves::SortedTroves;

pub mod sorted_troves_utils {
    use fuels::{
        accounts::fuel_crypto::rand::{self, Rng},
        prelude::WalletUnlocked,
    };

    use crate::utils::setup::{get_nominal_icr, set_nominal_icr_and_insert};

    use super::*;

    pub async fn assert_neighbors(
        sorted_troves: &SortedTroves<WalletUnlocked>,
        current: Identity,
        prev_id: Identity,
        next_id: Identity,
    ) {
        let next = sorted_troves_abi::get_next(&sorted_troves, current.clone()).await;
        assert_eq!(next.value, next_id);

        let prev = sorted_troves_abi::get_prev(&sorted_troves, current.clone()).await;
        assert_eq!(prev.value, prev_id);
    }

    pub async fn assert_in_order_from_head(
        sorted_troves: &SortedTroves<WalletUnlocked>,
        trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    ) {
        let mut count = 0;
        let size = sorted_troves_abi::get_size(sorted_troves).await.value;

        let mut current = sorted_troves_abi::get_first(sorted_troves).await.value;

        let mut next = sorted_troves_abi::get_next(sorted_troves, current.clone())
            .await
            .value;

        while next.clone() != Identity::Address([0; 32].into()) {
            let current_icr = get_nominal_icr(trove_manager, current.clone()).await.value;

            let next_icr = get_nominal_icr(trove_manager, next.clone()).await.value;

            assert!(
                current_icr >= next_icr,
                "ICR of current trove {} is less than next trove {}",
                current_icr,
                next_icr
            );

            current = next.clone();
            next = sorted_troves_abi::get_next(&sorted_troves, current.clone())
                .await
                .value
                .clone();

            count += 1;
        }

        assert_eq!(count, size - 1, "Insure that all nodes a traversed");
    }

    pub async fn assert_in_order_from_tail(
        sorted_troves: &SortedTroves<WalletUnlocked>,
        trove_manager: &MockTroveManagerContract<WalletUnlocked>,
    ) {
        let mut count = 0;
        let size = sorted_troves_abi::get_size(sorted_troves).await.value;

        let mut current = sorted_troves_abi::get_last(&sorted_troves).await.value;

        let mut prev = sorted_troves_abi::get_prev(&sorted_troves, current.clone())
            .await
            .value;

        while prev.clone() != Identity::Address([0; 32].into()) {
            let current_icr = get_nominal_icr(trove_manager, current.clone()).await.value;

            let prev_icr = get_nominal_icr(trove_manager, prev.clone()).await.value;

            assert!(current_icr <= prev_icr);

            current = prev.clone();
            prev = sorted_troves_abi::get_prev(&sorted_troves, current.clone())
                .await
                .value
                .clone();
            count += 1;
        }

        assert_eq!(count, size - 1, "Insure that all nodes a traversed");
    }

    pub async fn generate_random_nodes(
        trove_manager: &MockTroveManagerContract<WalletUnlocked>,
        sorted_troves: &SortedTroves<WalletUnlocked>,
        max_size: u64,
    ) -> (Vec<(Identity, u64)>, u64) {
        let mut count = 0;
        let mut rng = rand::thread_rng();

        let mut pairs: Vec<(Identity, u64)> = vec![];
        let mut avg_gas = 0;

        while count < max_size {
            let random_number = rng.gen::<u64>() % 10000;
            let random_address = rng.gen::<[u8; 32]>();

            pairs.push((
                Identity::Address(random_address.clone().into()),
                random_number.clone(),
            ));

            let gas_used = set_nominal_icr_and_insert(
                trove_manager,
                &sorted_troves,
                Identity::Address(random_address.into()),
                random_number,
                Identity::Address([0; 32].into()),
                Identity::Address([0; 32].into()),
            )
            .await
            .gas_used;
            avg_gas += gas_used;

            count += 1;
        }

        avg_gas /= max_size;

        return (pairs, avg_gas);
    }
}
