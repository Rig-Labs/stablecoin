use crate::data_structures::PRECISION;
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;
use fuels::types::U256;

abigen!(Contract(
    name = "RedstoneCore",
    abi = "contracts/mock-redstone-contract/out/debug/mock-redstone-contract-abi.json"
));

pub const DEFAULT_REDSTONE_PRICE_ID: U256 = U256::zero();

pub fn redstone_price_feed(prices: Vec<u64>) -> Vec<(U256, U256)> {
    let mut feed = Vec::with_capacity(prices.len());
    for price in prices {
        feed.push((U256::zero(), U256::from(price * PRECISION)));
    }
    feed
}

pub fn redstone_price_feed_with_id(price_id: U256, prices: Vec<u64>) -> Vec<(U256, U256)> {
    let mut feed = Vec::with_capacity(prices.len());
    for price in prices {
        feed.push((price_id, U256::from(price * PRECISION)));
    }
    feed
}

pub mod redstone_oracle_abi {

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        types::U256,
    };

    pub async fn read_prices<T: Account + Clone>(
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

    pub async fn write_prices<T: Account + Clone>(
        oracle: &RedstoneCore<T>,
        feed: Vec<(U256, U256)>,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .write_prices(feed)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn read_timestamp<T: Account + Clone>(oracle: &RedstoneCore<T>) -> CallResponse<u64> {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .read_timestamp()
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn set_timestamp<T: Account + Clone>(
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
