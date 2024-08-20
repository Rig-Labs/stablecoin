use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;
use fuels::types::U256;

abigen!(Contract(
    name = "RedstoneCore",
    abi = "contracts/mock-redstone-contract/out/debug/mock-redstone-contract-abi.json"
));

pub const REDSTONE_PRICE_ID: U256 = U256::zero();

pub mod redstone_oracle_abi {

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        types::U256,
    };

    pub async fn read_prices<T: Account>(
        oracle: &RedstoneCore<T>,
        price_feed_ids: Vec<U256>,
    ) -> CallResponse<Vec<U256>> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .read_prices(price_feed_ids)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn read_timestamp<T: Account>(oracle: &RedstoneCore<T>) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .read_timestamp()
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_timestamp<T: Account>(
        oracle: &RedstoneCore<T>,
        timestamp: u64,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .set_timestamp(timestamp)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }
}
