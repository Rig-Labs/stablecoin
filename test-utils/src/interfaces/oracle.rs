use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/mock-oracle-contract/out/debug/mock-oracle-contract-abi.json"
));

pub async fn set_price(oracle: &Oracle, price: u64) -> FuelCallResponse<()> {
    oracle.methods().set_price(price).call().await.unwrap()
}

pub async fn get_price(oracle: &Oracle) -> FuelCallResponse<u64> {
    oracle.methods().get_price().call().await.unwrap()
}
