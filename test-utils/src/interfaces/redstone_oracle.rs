use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "RedstoneCore",
    abi = "contracts/mock-redstone-contract/out/debug/mock-redstone-contract-abi.json"
));

pub mod redstone_oracle_abi {

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        types::{Bytes, U256},
    };

    pub async fn get_prices<T: Account>(
        oracle: &RedstoneCore<T>,
        price_feed_ids: Vec<U256>,
    ) -> CallResponse<(Vec<U256>, u64)> {
        let tx_params = TxPolicies::default().with_tip(1);
        let hex_str = "0101010101010101010101010101010101010101010101010101010101010101";

        let bytes = Bytes::from_hex_str(hex_str).unwrap();
        oracle
            .methods()
            .get_prices(price_feed_ids, bytes)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }
}
