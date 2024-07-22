use fuels::prelude::{abigen, ContractId};
use fuels::programs::responses::CallResponse;
use fuels::types::Identity;

abigen!(Contract(
    name = "SortedTroves",
    abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
));

pub mod sorted_troves_abi {
    use super::*;

    use fuels::{
        prelude::{Account, Error, TxPolicies},
        types::AssetId,
    };

    pub async fn initialize<T: Account>(
        sorted_troves: &SortedTroves<T>,
        max_size: u64,
        protocol_manager: ContractId,
        borrow_opperations: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = sorted_troves
            .methods()
            .set_params(max_size, protocol_manager, borrow_opperations)
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn insert<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
        icr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: AssetId,
    ) -> CallResponse<()> {
        sorted_troves
            .methods()
            .insert(id, icr, prev_id, next_id, asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: AssetId,
        trove_manager: ContractId,
    ) -> CallResponse<()> {
        sorted_troves
            .methods()
            .add_asset(asset.into(), trove_manager)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_first<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .methods()
            .get_first(asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn get_last<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .methods()
            .get_last(asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn get_size<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: AssetId,
    ) -> CallResponse<u64> {
        sorted_troves
            .methods()
            .get_size(asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn get_next<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .methods()
            .get_next(id, asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn get_prev<T: Account>(
        sorted_troves: &SortedTroves<T>,
        id: Identity,
        asset: AssetId,
    ) -> CallResponse<Identity> {
        sorted_troves
            .methods()
            .get_prev(id, asset.into())
            .call()
            .await
            .unwrap()
    }
}
