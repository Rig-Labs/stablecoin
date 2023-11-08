use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;
abigen!(Contract(
    name = "HintHelper",
    abi = "contracts/hint-helper-contract/out/debug/hint-helper-contract-abi.json"
));

pub mod hint_helper_abi {
    use super::*;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use fuels::prelude::Account;
    use fuels::{
        prelude::{ContractId, Error, TxParameters},
        types::{AssetId, Identity},
    };

    pub async fn initialize<T: Account>(
        hint_helper: &HintHelper<T>,

        sorted_troves: ContractId,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = hint_helper
            .methods()
            .initialize(sorted_troves)
            .tx_params(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_approx_hint<T: Account>(
        hint_helper: &HintHelper<T>,
        trove_manager: &TroveManagerContract<T>,
        sorted_troves: &SortedTroves<T>,
        asset_id: &AssetId,
        cr: u64,
        num_itterations: u64,
        random_seed: u64,
    ) -> FuelCallResponse<(Identity, u64, u64)> {
        hint_helper
            .methods()
            .get_approx_hint(
                asset_id.clone(),
                trove_manager.contract_id(),
                cr,
                num_itterations,
                random_seed,
            )
            .with_contracts(&[sorted_troves, trove_manager])
            .call()
            .await
            .unwrap()
    }
}
