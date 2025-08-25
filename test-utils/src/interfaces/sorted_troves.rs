use fuels::prelude::{abigen, ContractId};
use fuels::programs::responses::CallResponse;
use fuels::types::Identity;

abigen!(Contract(
    name = "SortedTroves",
    abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
));

pub mod sorted_troves_abi {
    use crate::{
        data_structures::ContractInstance, interfaces::trove_manager::TroveManagerContract,
    };

    use super::*;

    use fuels::{
        prelude::{Account, Error, TxPolicies},
        types::AssetId,
    };

    pub async fn initialize<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        max_size: u64,
        protocol_manager: ContractId,
        borrow_opperations: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = sorted_troves
            .contract
            .methods()
            .set_params(max_size, protocol_manager, borrow_opperations)
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn insert<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        id: Identity,
        icr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: AssetId,
    ) -> CallResponse<()> {
        sorted_troves
            .contract
            .methods()
            .insert(id, icr, prev_id, next_id, asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        asset: AssetId,
        trove_manager: ContractId,
    ) -> CallResponse<()> {
        sorted_troves
            .contract
            .methods()
            .add_asset(asset.into(), trove_manager)
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_first<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .contract
            .methods()
            .get_first(asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_last<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .contract
            .methods()
            .get_last(asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_size<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        asset: AssetId,
    ) -> CallResponse<u64> {
        sorted_troves
            .contract
            .methods()
            .get_size(asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_next<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        id: Identity,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .contract
            .methods()
            .get_next(id, asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_prev<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        id: Identity,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .contract
            .methods()
            .get_prev(id, asset.into())
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_max_size<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
    ) -> CallResponse<u64> {
        sorted_troves
            .contract
            .methods()
            .get_max_size()
            .with_contract_ids(&[sorted_troves.implementation_id.into()])
            .call()
            .await
            .unwrap()
    }

    pub async fn contains<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        id: Identity,
        asset: AssetId,
    ) -> CallResponse<bool> {
        sorted_troves
            .contract
            .methods()
            .contains(id, asset.into())
            .with_contract_ids(&[
                sorted_troves.implementation_id.into(),
                sorted_troves.contract.contract_id().into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn find_insert_position<T: Account + Clone>(
        sorted_troves: &ContractInstance<SortedTroves<T>>,
        trove_manager: &TroveManagerContract<T>,
        icr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: AssetId,
    ) -> CallResponse<(Identity, Identity)> {
        sorted_troves
            .contract
            .methods()
            .find_insert_position(icr, prev_id, next_id, asset.into())
            .with_contracts(&[trove_manager])
            .with_contract_ids(&[
                sorted_troves.implementation_id.into(),
                sorted_troves.contract.contract_id().into(),
                trove_manager.contract_id().into(),
            ])
            .call()
            .await
            .unwrap()
    }
}
