use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "FPTToken",
    abi = "contracts/fpt-token-contract/out/debug/fpt-token-contract-abi.json"
));

pub mod fpt_token_abi {
    use fuels::{
        prelude::{Account, AssetId, CallParameters, Error, LogDecoder, TxParameters},
        types::ContractId,
    };

    use crate::{setup::common::wait, interfaces::vesting};

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &FPTToken<T>,
        mut name: String,
        mut symbol: String,
        vesting_contract: ContractId,
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
            .initialize(
                config,
                vesting_contract
            )
            .tx_params(tx_params)
            .append_variable_outputs(10)
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

    pub async fn total_supply<T: Account>(instance: &FPTToken<T>) -> FuelCallResponse<u64> {
        instance.methods().total_supply().call().await.unwrap()
    }

    pub async fn get_vesting_contract<T: Account>(instance: &FPTToken<T>) -> FuelCallResponse<ContractId> {
        instance.methods().get_vesting_contract().call().await.unwrap()
    }
}
