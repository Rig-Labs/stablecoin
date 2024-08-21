use fuels::{prelude::*, types::Bits256};
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        oracle::{oracle_abi, Oracle, ORACLE_TIMEOUT},
        pyth_oracle::{pyth_oracle_abi, PythCore, PythPrice, PythPriceFeed},
        redstone_oracle::{redstone_oracle_abi, redstone_price_feed, RedstoneCore},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_oracle},
};

async fn setup() -> (
    Oracle<WalletUnlocked>,
    PythCore<WalletUnlocked>,
    RedstoneCore<WalletUnlocked>,
    WalletUnlocked,
) {
    let block_time = 1u32; // seconds
    let config = NodeConfig {
        block_production: Trigger::Interval {
            block_time: std::time::Duration::from_secs(block_time.into()),
        },
        ..NodeConfig::default()
    };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), Some(1), Some(1_000_000_000)),
        Some(config),
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();

    let pyth = deploy_mock_pyth_oracle(&wallet).await;
    let redstone = deploy_mock_redstone_oracle(&wallet).await;
    let oracle = deploy_oracle(
        &wallet,
        pyth.contract_id().into(),
        redstone.contract_id().into(),
    )
    .await;

    (oracle, pyth, redstone, wallet)
}

pub fn pyth_feed(price: u64, unix_timestamp: u64) -> Vec<(Bits256, PythPriceFeed)> {
    let tai64_offset = 4611686018427387904;
    // Leap seconds offset (as of 2023, TAI is ahead by 37 seconds)
    let leap_seconds = 37;
    vec![(
        Bits256::zeroed(),
        PythPriceFeed {
            price: PythPrice {
                price: price * PRECISION,
                publish_time: unix_timestamp + tai64_offset + leap_seconds,
            },
        },
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    mod live_pyth {
        use super::*;

        #[ignore]
        #[tokio::test]
        async fn price() {
            let (oracle, pyth, redstone, wallet) = setup().await;
            let provider = wallet.try_provider().unwrap();
            let timestamp = provider
                .latest_block_time()
                .await
                .unwrap()
                .unwrap()
                .timestamp() as u64;

            let expected_price = 1 * PRECISION;

            pyth_oracle_abi::update_price_feeds(&pyth, pyth_feed(1, timestamp)).await;

            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price, price);
        }

        #[ignore]
        #[tokio::test]
        async fn fallback_to_last_price() {
            let (_oracle, _pyth, _redstone, _wallet) = setup().await;
        }
    }

    mod pyth_timeout {
        use super::*;

        #[ignore]
        #[tokio::test]
        async fn live_redstone() {
            let (_oracle, _pyth, _redstone, _wallet) = setup().await;
        }

        #[ignore]
        #[tokio::test]
        async fn live_redstone_fallback_to_last_price() {
            let (_oracle, _pyth, _redstone, _wallet) = setup().await;
        }

        mod redstone_timeout {
            use super::*;

            mod pyth_timestamp_more_recent {
                use super::*;

                #[ignore]
                #[tokio::test]
                async fn price() {
                    let (_oracle, _pyth, _redstone, _wallet) = setup().await;
                }

                #[ignore]
                #[tokio::test]
                async fn fallback_last_price() {
                    let (_oracle, _pyth, _redstone, _wallet) = setup().await;
                }
            }

            mod redstone_timestamp_more_recent {
                use super::*;

                #[ignore]
                #[tokio::test]
                async fn price() {
                    let (_oracle, _pyth, _redstone, _wallet) = setup().await;
                }

                #[ignore]
                #[tokio::test]
                async fn fallback_last_price() {
                    let (_oracle, _pyth, _redstone, _wallet) = setup().await;
                }
            }
        }
    }
}
