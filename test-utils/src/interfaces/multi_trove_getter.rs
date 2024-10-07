use fuels::prelude::abigen;

use fuels::programs::responses::CallResponse;
abigen!(Contract(
    name = "MultiTroveGetter",
    abi = "contracts/multi-trove-getter-contract/out/debug/multi-trove-getter-contract-abi.json"
));

pub mod multi_trove_getter_abi {
    use super::*;
    use crate::interfaces::{sorted_troves::SortedTroves, trove_manager::TroveManagerContract};
    use fuels::prelude::Account;
    use fuels::types::AssetId;

    pub async fn get_multiple_sorted_troves<T: Account>(
        multi_trove_getter: &MultiTroveGetter<T>,
        trove_manager: &TroveManagerContract<T>,
        sorted_troves: &SortedTroves<T>,
        asset_id: &AssetId,
        start_indx: u64,
        count: u8,
    ) -> CallResponse<Vec<CombinedTroveData>> {
        multi_trove_getter
            .methods()
            .get_multiple_sorted_troves(
                trove_manager.contract_id(),
                asset_id.clone(),
                start_indx,
                count,
            )
            .with_contracts(&[sorted_troves, trove_manager])
            .call()
            .await
            .unwrap()
    }
}
