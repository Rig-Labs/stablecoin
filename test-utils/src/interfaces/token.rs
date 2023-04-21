use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "Token",
    abi = "contracts/token-contract/out/debug/token-contract-abi.json"
));

pub mod token_abi {
    use fuels::prelude::{Account, LogDecoder, TxParameters};

    use crate::setup::common::wait;

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &Token<T>,
        amount: u64,
        admin: &Identity,
        mut name: String,
        mut symbol: String,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        name.push_str(" ".repeat(32 - name.len()).as_str());
        symbol.push_str(" ".repeat(8 - symbol.len()).as_str());

        let config = TokenInitializeConfig {
            name: fuels::types::SizedAsciiString::<32>::new(name.clone()).unwrap(),
            symbol: fuels::types::SizedAsciiString::<8>::new(symbol.clone()).unwrap(),
            decimals: 6,
        };

        let res = instance
            .methods()
            .initialize(config, amount, admin.clone())
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

    pub async fn mint_to_id<T: Account>(
        instance: &Token<T>,
        amount: u64,
        admin: Identity,
    ) -> FuelCallResponse<()> {
        instance
            .methods()
            .mint_to_id(amount, admin)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }
}

pub mod token_utils {}
