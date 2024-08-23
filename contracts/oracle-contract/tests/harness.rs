use fuels::prelude::*;
use fuels::types::U256;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        oracle::{oracle_abi, Oracle, ORACLE_TIMEOUT},
        pyth_oracle::{pyth_oracle_abi, pyth_price_feed_with_time, PythCore, PYTH_TIMESTAMP},
        redstone_oracle::{redstone_oracle_abi, RedstoneCore},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_oracle},
};

async fn setup() -> (
    Oracle<WalletUnlocked>,
    PythCore<WalletUnlocked>,
    RedstoneCore<WalletUnlocked>,
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

    (oracle, pyth, redstone)
}

fn redstone_feed(price: u64) -> Vec<(U256, U256)> {
    vec![(U256::zero(), U256::from(price * PRECISION))]
}

#[cfg(test)]
mod tests {
    use super::*;

    mod live_pyth {
        use super::*;

        #[tokio::test]
        async fn price() {
            let (oracle, pyth, redstone) = setup().await;
            let expected_price = 1 * PRECISION;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
            )
            .await;

            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn fallback_to_last_price() {
            let (oracle, pyth, redstone) = setup().await;
            let expected_price = 1 * PRECISION;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price, price);

            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP - 1),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price, price);
        }
    }

    mod pyth_timeout {
        use super::*;

        #[tokio::test]
        async fn live_redstone() {
            let (oracle, pyth, redstone) = setup().await;
            let expected_price = 1 * PRECISION;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP),
            )
            .await;
            redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP + 1).await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price * 3, price);
        }

        #[ignore = "Unreachable by logic so skip test but leave for acknowledgement of case"]
        #[tokio::test]
        async fn live_redstone_fallback_to_last_price() {}

        mod redstone_timeout {
            use super::*;

            mod pyth_timestamp_more_recent {
                use super::*;

                #[tokio::test]
                async fn price() {
                    let (oracle, pyth, redstone) = setup().await;
                    let expected_price = 1 * PRECISION;

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP + 1),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price * 2, price);
                }

                #[tokio::test]
                async fn fallback_last_price() {
                    let (oracle, pyth, redstone) = setup().await;
                    let expected_price = 1 * PRECISION;

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);
                }
            }

            mod redstone_timestamp_more_recent {
                use super::*;

                #[tokio::test]
                async fn price() {
                    let (oracle, pyth, redstone) = setup().await;
                    let expected_price = 1 * PRECISION;

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP + 1).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price * 3, price);
                }

                #[tokio::test]
                async fn fallback_last_price() {
                    let (oracle, pyth, redstone) = setup().await;
                    let expected_price = 1 * PRECISION;

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price, price);
                }
            }
        }
    }
}
