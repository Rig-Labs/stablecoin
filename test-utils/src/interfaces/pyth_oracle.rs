use crate::data_structures::PRECISION;
use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;
use fuels::types::Bits256;
use lazy_static::lazy_static;

abigen!(Contract(
    name = "PythCore",
    abi = "contracts/mock-pyth-contract/out/debug/mock-pyth-contract-abi.json"
));

pub const PYTH_PRICE_ID: Bits256 = Bits256([0; 32]);
lazy_static! {
    pub static ref PYTH_FEEDS: Vec<(Bits256, PythPriceFeed)> = vec![(
        Bits256::zeroed(),
        PythPriceFeed {
            price: PythPrice {
                price: 1 * PRECISION,
                publish_time: 1724166967
            }
        }
    )];
}

pub mod pyth_oracle_abi {

    use super::*;
    use fuels::prelude::{Account, TxPolicies};

    pub async fn price<T: Account>(
        oracle: &PythCore<T>,
        price_feed_id: Bits256,
    ) -> CallResponse<PythPrice> {
        let tx_params = TxPolicies::default().with_tip(1);
        oracle
            .methods()
            .price(price_feed_id)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn update_price_feeds<T: Account>(
        oracle: &PythCore<T>,
        feeds: Vec<(Bits256, PythPriceFeed)>,
    ) {
        let tx_params = TxPolicies::default().with_tip(1);

        oracle
            .methods()
            .update_price_feeds(feeds)
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap();
    }
}
