use fuels::prelude::abigen;

use fuels::programs::responses::CallResponse;
abigen!(Contract(
    name = "HintHelper",
    abi = "contracts/hint-helper-contract/out/debug/hint-helper-contract-abi.json"
));

pub mod hint_helper_abi {
    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::trove_manager::TroveManagerContract;
    use fuels::prelude::Account;
    use fuels::{
        prelude::{ContractId, Error, TxPolicies},
        types::{AssetId, Identity},
    };

    pub async fn initialize<T: Account>(
        hint_helper: &HintHelper<T>,

        sorted_troves: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = hint_helper
            .methods()
            .initialize(sorted_troves)
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_approx_hint<T: Account>(
        hint_helper: &HintHelper<T>,
        trove_manager: &ContractInstance<TroveManagerContract<T>>,
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        asset_id: &AssetId,
        cr: u64,
        num_itterations: u64,
        random_seed: u64,
    ) -> CallResponse<(Identity, u64, u64)> {
        hint_helper
            .methods()
            .get_approx_hint(
                asset_id.clone(),
                trove_manager.contract.contract_id(),
                cr,
                num_itterations,
                random_seed,
            )
            .with_contracts(&[&sorted_troves.contract, &trove_manager.contract])
            .with_contract_ids(&[
                sorted_troves.contract.contract_id().into(),
                sorted_troves.implementation_id.into(),
                trove_manager.contract.contract_id().into(),
                trove_manager.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }
}
