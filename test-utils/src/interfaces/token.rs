use fuels::{
    prelude::{abigen, WalletUnlocked},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "Token",
    abi = "contracts/token-contract/out/debug/token-contract-abi.json"
));

pub mod token_abi {
    use super::*;
    pub async fn initialize(
        instance: &Token,
        amount: u64,
        admin: &Identity,
        mut name: String,
        mut symbol: String,
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
            .initialize(config, amount, admin.clone())
            .call()
            .await
            .unwrap()
    }

    pub async fn mint_to_id(
        instance: &Token,
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
