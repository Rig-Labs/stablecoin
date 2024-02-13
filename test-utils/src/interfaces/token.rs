use fuels::programs::call_utils::TxDependencyExtension;
use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "Token",
    abi = "contracts/token-contract/out/debug/token-contract-abi.json"
));

pub mod token_abi {
    use super::*;
    use fuels::prelude::{Account, Error, TxPolicies};

    pub async fn initialize<T: Account>(
        instance: &Token<T>,
        amount: u64,
        admin: &Identity,
        mut name: String,
        mut symbol: String,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_gas_price(1);

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
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
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
