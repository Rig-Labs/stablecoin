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
        protocol_manager: ContractId,
        borrow_opperations: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = sorted_troves
            .methods()
            .set_params(max_size, protocol_manager, borrow_opperations)
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
        asset: ContractId,
    ) -> FuelCallResponse<()> {
        sorted_troves
            .methods()
            .insert(id, icr, prev_id, next_id, asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: ContractId,
        trove_manager: ContractId,
    ) -> FuelCallResponse<()> {
        sorted_troves
            .methods()
            .add_asset(asset, trove_manager)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_first<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_first(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_last<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: ContractId,
    ) -> FuelCallResponse<Identity> {
        sorted_troves
            .methods()
            .get_last(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_size<T: Account>(
        sorted_troves: &SortedTroves<T>,
        asset: ContractId,
    ) -> FuelCallResponse<u64> {
        sorted_troves
            .methods()
            .get_size(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_next<T: Account>(
        sorted_troves: &SortedTroves<T>,
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

    pub async fn get_prev<T: Account>(
        sorted_troves: &SortedTroves<T>,
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
