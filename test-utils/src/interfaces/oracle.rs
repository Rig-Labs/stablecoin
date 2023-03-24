use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/mock-oracle-contract/out/debug/mock-oracle-contract-abi.json"
));

pub mod oracle_abi {

    use fuels::prelude::{LogDecoder, TxParameters};

    use crate::setup::common::wait;

    use super::*;

    pub async fn set_price(oracle: &Oracle, price: u64) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = oracle
            .methods()
            .set_price(price)
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

    pub async fn get_price(oracle: &Oracle) -> FuelCallResponse<u64> {
        oracle.methods().get_price().call().await.unwrap()
    }
}
