use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "PythCore",
    abi = "contracts/mock-pyth-contract/out/debug/mock-pyth-contract-abi.json"
));

pub mod pyth_oracle_abi {

    use super::*;
    use fuels::{
        prelude::{Account, TxPolicies},
        types::Bits256,
    };

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
