use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "USDFToken",
    abi = "contracts/usdf-token-contract/out/debug/usdf-token-contract-abi.json"
));

pub mod usdf_token_abi {
    use fuels::{
        prelude::{Account, AssetId, CallParameters, Error, LogDecoder, TxParameters},
        types::ContractId,
    };

    use crate::setup::common::wait;

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &USDFToken<T>,
        mut name: String,
        mut symbol: String,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrow_operations: Identity,
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
                protocol_manager,
                stability_pool.clone(),
                borrow_operations.clone(),
            )
            .tx_params(tx_params)
            .call()
            .await;

        return res.unwrap();

        // TODO: remove this workaround
        match res {
            Ok(res) => res,
            Err(_) => {
                wait();
                return FuelCallResponse::new((), vec![], LogDecoder::default());
            }
        }
    }

    pub async fn mint<T: Account>(
        instance: &USDFToken<T>,
        amount: u64,
        address: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        instance
            .methods()
            .mint(amount, address)
            .append_variable_outputs(1)
            .call()
            .await
    }

    pub async fn burn<T: Account>(
        usdf_token: &USDFToken<T>,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().set_gas_price(1);
        let usdf_asset_id = AssetId::from(*usdf_token.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(usdf_asset_id);

        usdf_token
            .methods()
            .burn()
            .call_params(call_params)
            .unwrap()
            .tx_params(tx_params)
            .append_variable_outputs(1)
            .call()
            .await
    }

    pub async fn total_supply<T: Account>(instance: &USDFToken<T>) -> FuelCallResponse<u64> {
        instance.methods().total_supply().call().await.unwrap()
    }
}
