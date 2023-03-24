use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "USDFToken",
    abi = "contracts/usdf-token-contract/out/debug/usdf-token-contract-abi.json"
));

pub mod usdf_token_abi {
    use fuels::prelude::{AssetId, CallParameters, Error, TxParameters};

    use super::*;
    pub async fn initialize(
        instance: &USDFToken,
        mut name: String,
        mut symbol: String,
        trove_manager: Identity,
        stability_pool: Identity,
        borrow_operations: Identity,
    ) -> FuelCallResponse<()> {
        name.push_str(" ".repeat(32 - name.len()).as_str());
        symbol.push_str(" ".repeat(8 - symbol.len()).as_str());

        let config = TokenInitializeConfig {
            name: fuels::types::SizedAsciiString::<32>::new(name).unwrap(),
            symbol: fuels::types::SizedAsciiString::<8>::new(symbol).unwrap(),
            decimals: 6,
        };

        instance
            .methods()
            .initialize(config, trove_manager, stability_pool, borrow_operations)
            .call()
            .await
            .unwrap()
    }

    pub async fn mint(
        instance: &USDFToken,
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

    pub async fn burn(usdf_token: &USDFToken, amount: u64) -> Result<FuelCallResponse<()>, Error> {
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

    pub async fn total_supply(instance: &USDFToken) -> FuelCallResponse<u64> {
        instance.methods().total_supply().call().await.unwrap()
    }
}
