use fuels::prelude::abigen;
use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "Oracle",
    abi = "contracts/mock-oracle-contract/out/debug/mock-oracle-contract-abi.json"
));

pub mod oracle_abi {

    use super::*;
    use fuels::prelude::{Account, TxParameters};

    pub async fn set_price<T: Account>(oracle: &Oracle<T>, price: u64) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = oracle
            .methods()
            .set_price(price)
            .tx_params(tx_params)
            .call()
            .await;

        return res.unwrap();
        // TODO: remove this workaround
    }

    pub async fn get_price<T: Account>(oracle: &Oracle<T>) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::default().with_gas_price(1);
        oracle
            .methods()
            .get_price()
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }
}
