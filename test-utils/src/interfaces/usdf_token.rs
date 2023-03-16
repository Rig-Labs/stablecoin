use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "USDFToken",
    abi = "contracts/usdf-token-contract/out/debug/usdf-token-contract-abi.json"
));

pub mod usdf_token_abi {
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

    pub async fn mint(instance: &USDFToken, amount: u64, admin: Identity) -> FuelCallResponse<()> {
        instance
            .methods()
            .mint(amount, admin)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }
}
