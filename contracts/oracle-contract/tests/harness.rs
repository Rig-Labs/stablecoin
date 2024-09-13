use fuels::prelude::*;
use fuels::types::U256;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        oracle::{oracle_abi, Oracle, ORACLE_TIMEOUT},
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed_with_time, PythCore, DEFAULT_PYTH_PRICE_ID,
            PYTH_TIMESTAMP,
        },
        redstone_oracle::{redstone_oracle_abi, RedstoneCore, DEFAULT_REDSTONE_PRICE_ID},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_oracle},
};

const PYTH_PRECISION: u8 = 12;
const REDSTONE_PRECISION: u8 = 6;

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
        PYTH_PRECISION,
        DEFAULT_PYTH_PRICE_ID,
        redstone.contract_id().into(),
        REDSTONE_PRECISION,
        DEFAULT_REDSTONE_PRICE_ID,
        true,
    )
    .await;

    (oracle, pyth, redstone)
}

fn redstone_feed(price: u64) -> Vec<(U256, U256)> {
    vec![(DEFAULT_REDSTONE_PRICE_ID, U256::from(price * PRECISION))]
}

fn convert_precision(price: u64, current_precision: u32) -> u64 {
    let mut adjusted_price = 0;
    if current_precision > 9 {
        adjusted_price = price / (10_u64.pow(current_precision - 9));
    } else if current_precision < 9 {
        adjusted_price = price * 10_u64.pow(9 - current_precision);
    } else {
        adjusted_price = price;
    }

    adjusted_price
}

#[cfg(test)]
mod tests {
    use super::*;

    mod live_pyth {
        use super::*;

        #[tokio::test]
        async fn price() {
            let (oracle, pyth, redstone) = setup().await;
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

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
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

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
            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone =
                convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price_pyth, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP),
            )
            .await;
            redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP + 1).await;
            let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

            assert_eq!(expected_price_redstone, price);
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
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

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
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

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
                    let expected_price_pyth =
                        convert_precision(1 * PRECISION, PYTH_PRECISION.into());
                    let expected_price_redstone =
                        convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &redstone).await.value;

                    assert_eq!(expected_price_pyth, price);

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

                    let redstone_price = redstone_oracle_abi::read_prices(
                        &redstone,
                        vec![DEFAULT_REDSTONE_PRICE_ID],
                    )
                    .await
                    .value[0]
                        .as_u64();

                    let converted_redstone_price =
                        convert_precision(redstone_price, REDSTONE_PRECISION.into());

                    assert_eq!(expected_price_redstone, converted_redstone_price);
                    assert_eq!(expected_price_redstone, price);
                }

                #[tokio::test]
                async fn fallback_last_price() {
                    let (oracle, pyth, redstone) = setup().await;
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

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
