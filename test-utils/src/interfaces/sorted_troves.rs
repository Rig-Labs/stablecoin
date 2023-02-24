use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

abigen!(Contract(
    name = "SortedTroves",
    abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
));

pub async fn get_first(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
    sorted_troves.methods().get_first().call().await.unwrap()
}

pub async fn get_last(sorted_troves: &SortedTroves) -> FuelCallResponse<Identity> {
    sorted_troves.methods().get_last().call().await.unwrap()
}

pub async fn get_size(sorted_troves: &SortedTroves) -> FuelCallResponse<u64> {
    sorted_troves.methods().get_size().call().await.unwrap()
}

pub async fn get_next(sorted_troves: &SortedTroves, id: Identity) -> FuelCallResponse<Identity> {
    sorted_troves.methods().get_next(id).call().await.unwrap()
}

pub async fn get_prev(sorted_troves: &SortedTroves, id: Identity) -> FuelCallResponse<Identity> {
    sorted_troves.methods().get_prev(id).call().await.unwrap()
}
