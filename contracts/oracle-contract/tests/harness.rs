use fuels::types::U256;
use fuels::{prelude::*, types::Identity};
use test_utils::data_structures::ContractInstance;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        oracle::{oracle_abi, Oracle, RedstoneConfig, ORACLE_TIMEOUT},
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed_with_time, PythCore, DEFAULT_PYTH_PRICE_ID,
            PYTH_TIMESTAMP,
        },
        redstone_oracle::{redstone_oracle_abi, RedstoneCore, DEFAULT_REDSTONE_PRICE_ID},
        stork_oracle::{stork_oracle_abi, StorkCore, StorkConfig, DEFAULT_STORK_FEED_ID, NS_TO_SECONDS},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_oracle},
};

const PYTH_PRECISION: u8 = 12;
const REDSTONE_PRECISION: u32 = 6;
const DEFAULT_FUEL_VM_DECIMALS: u32 = 9;

async fn setup(
    redstone_precision: u32,
    fuel_vm_decimals: u32,
    initialize_pyth: bool,
    initialize_redstone: bool,
    initialize_stork: bool,
) -> (
    ContractInstance<Oracle<WalletUnlocked>>,
    Option<PythCore<WalletUnlocked>>,
    Option<RedstoneCore<WalletUnlocked>>,
    Option<StorkCore<WalletUnlocked>>,
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

    if initialize_pyth {
        let pyth = deploy_mock_pyth_oracle(&wallet).await;

        let oracle = deploy_oracle(
            &wallet,
            pyth.contract_id().into(),
            DEFAULT_PYTH_PRICE_ID,
            fuel_vm_decimals,
            true,
            Identity::Address(wallet.address().into()),
        )
        .await;

        return (oracle, Some(pyth), None, None);
    }

    if initialize_redstone {
        let redstone = deploy_mock_redstone_oracle(&wallet).await;

        oracle_abi::set_redstone_config(
            &oracle,
            &redstone,
            RedstoneConfig {
                contract_id: redstone.contract_id().into(),
                price_id: DEFAULT_REDSTONE_PRICE_ID,
                precision: redstone_precision,
            },
        )
        .await
        .unwrap();

        return (oracle, pyth, Some(redstone));
    }

    if initialize_stork {
        let stork = deploy_mock_stork_oracle(&wallet).await;

        oracle_abi::set_stork_config(
            &oracle,
            &stork,
            StorkConfig {
                contract_id: stork.contract_id().into(),
                price_id: DEFAULT_STORK_FEED_ID,
                precision: fuel_vm_decimals,
            },
        )
        .await
        .unwrap();

        return (oracle, pyth, None, Some(stork));
    }

    (oracle, pyth, None, None)
}

fn redstone_feed(price: u64) -> Vec<(U256, U256)> {
    vec![(DEFAULT_REDSTONE_PRICE_ID, U256::from(price * PRECISION))]
}

fn convert_precision(price: u64, current_precision: u32) -> u64 {
    let adjusted_price;
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
            let (oracle, pyth, redstone) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            )
            .await;

            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.unwrap().clone()))
                .await
                .value;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn fallback_to_last_price() {
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            let redstone = redstone_wrapped.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            assert_eq!(expected_price, price);

            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP - 1, PYTH_PRECISION.into()),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            assert_eq!(expected_price, price);
        }
    }

    mod pyth_timeout {
        use super::*;

        #[tokio::test]
        async fn live_redstone() {
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;

            let redstone = redstone_wrapped.unwrap();
            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone =
                convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            )
            .await;
            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            assert_eq!(expected_price_pyth, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            )
            .await;
            redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP + 1).await;
            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

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
                    let (oracle, pyth, redstone_wrapped) =
                        setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

                    let redstone = redstone_wrapped.unwrap();

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price * 2, price);
                }

                #[tokio::test]
                async fn fallback_last_price() {
                    let (oracle, pyth, redstone_wrapped) =
                        setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

                    let redstone = redstone_wrapped.unwrap();

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;

                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price, price);
                }
            }

            mod redstone_timestamp_more_recent {
                use super::*;

                #[tokio::test]
                async fn price() {
                    let (oracle, pyth, redstone_wrapped) =
                        setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
                    let expected_price_pyth =
                        convert_precision(1 * PRECISION, PYTH_PRECISION.into());
                    let expected_price_redstone =
                        convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

                    let redstone = redstone_wrapped.unwrap();

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price_pyth, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP + 1).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

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
                    let (oracle, pyth, redstone_wrapped) =
                        setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

                    let redstone = redstone_wrapped.unwrap();

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    )
                    .await;
                    redstone_oracle_abi::write_prices(&redstone, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;
                    let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                        .await
                        .value;

                    assert_eq!(expected_price, price);
                }
            }
        }
    }

    mod confidence_check {
        use test_utils::interfaces::pyth_oracle::pyth_price_feed_with_confidence;

        use super::*;

        #[tokio::test]
        async fn price_within_confidence() {
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
            let price = 1000 * PRECISION;
            let confidence = price / 100; // 1% confidence, which is within the 4% threshold

            let redstone = redstone_wrapped.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_confidence(
                    price,
                    PYTH_TIMESTAMP,
                    confidence,
                    PYTH_PRECISION.into(),
                ),
            )
            .await;
            let result = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            assert_eq!(convert_precision(price, PYTH_PRECISION.into()), result);
        }

        #[tokio::test]
        async fn price_outside_confidence_good_redstone() {
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;

            let redstone = redstone_wrapped.unwrap();
            let pyth_price = 100 * PRECISION;
            let pyth_confidence = pyth_price / 20; // 5% confidence, which is outside the 4% threshold
            let redstone_price = 105 * PRECISION;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_confidence(
                    pyth_price,
                    PYTH_TIMESTAMP,
                    pyth_confidence,
                    PYTH_PRECISION.into(),
                ),
            )
            .await;
            redstone_oracle_abi::write_prices(&redstone, redstone_feed(redstone_price / PRECISION))
                .await;
            redstone_oracle_abi::set_timestamp(&redstone, PYTH_TIMESTAMP).await;

            let result = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            assert_eq!(
                convert_precision(redstone_price, REDSTONE_PRECISION.into()),
                result
            );
        }
    }

    mod fuel_vm_decimals {
        use super::*;

        #[tokio::test]
        async fn test_with_fuel_vm_decimals_8() {
            let fuel_vm_decimals = 8;
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, fuel_vm_decimals, true).await;

            let redstone = redstone_wrapped.unwrap();

            // Set a price of $5000 for 1 unit of the asset
            let pyth_price = 5000;
            let pyth_timestamp = PYTH_TIMESTAMP;

            oracle_abi::set_debug_timestamp(&oracle, pyth_timestamp).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(pyth_price, pyth_timestamp, 9),
            )
            .await;

            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            // Expected price calculation:
            // 1. Convert Pyth price to 8 decimal precision: 5000 * 10^8 = 500_000_000_000
            // 2. Multiply by 10 because 1_000_000_000 units with 8 decimals is 10 units of the asset
            let expected_price = 5000 * PRECISION * 10;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn test_with_fuel_vm_decimals_12() {
            let fuel_vm_decimals = 12;
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, fuel_vm_decimals, true).await;

            let redstone = redstone_wrapped.unwrap();

            // Set a price of $5000 for 1 unit of the asset
            let pyth_price = 5000;
            let pyth_timestamp = PYTH_TIMESTAMP;

            oracle_abi::set_debug_timestamp(&oracle, pyth_timestamp).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_time(pyth_price, pyth_timestamp, 9),
            )
            .await;

            let price = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;

            // Expected price calculation:
            // 1. Convert Pyth price to 12 decimal precision: 5000 * 10^12 = 5_000_000_000_000_000
            // 2. Divide by 1000 because 1_000_000_000 units with 12 decimals is 0.001 units of the asset
            let expected_price = 5000 * PRECISION / 1000;

            assert_eq!(expected_price, price);
        }
    }

    mod pyth_exponent_changes {
        use test_utils::interfaces::pyth_oracle::pyth_price_feed_with_confidence;

        use super::*;

        #[tokio::test]
        async fn test_pyth_price_adjustment_different_exponents() {
            let (oracle, pyth, redstone_wrapped) =
                setup(REDSTONE_PRECISION, DEFAULT_FUEL_VM_DECIMALS, true).await;
            let expected_price = 1 * PRECISION; // $1.00 with 9 decimal places

            let redstone = redstone_wrapped.unwrap();

            // Test with exponent 9
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_confidence(1_000_000_000, PYTH_TIMESTAMP, 0, 9),
            )
            .await;
            let price_exp_9 = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;
            assert_eq!(expected_price, price_exp_9);

            // Test with exponent 6
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + 1).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_confidence(1_000_000, PYTH_TIMESTAMP + 1, 0, 6),
            )
            .await;
            let price_exp_6 = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;
            assert_eq!(expected_price, price_exp_6);

            // Test with exponent 12
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + 2).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth,
                pyth_price_feed_with_confidence(1_000_000_000_000, PYTH_TIMESTAMP + 2, 0, 12),
            )
            .await;
            let price_exp_12 = oracle_abi::get_price(&oracle, &pyth, &Some(redstone.clone()))
                .await
                .value;
            assert_eq!(expected_price, price_exp_12);

            // Assert that all prices are equal
            assert_eq!(price_exp_9, price_exp_6);
            assert_eq!(price_exp_9, price_exp_12);
            assert_eq!(price_exp_6, price_exp_12);
        }
    }

    mod stork_oracle {
        use super::*;
        use test_utils::interfaces::stork_oracle::{stork_oracle_abi, StorkCore};

        async fn setup_with_stork(fuel_vm_decimals: u32) -> (
            ContractInstance<Oracle<WalletUnlocked>>,
            PythCore<WalletUnlocked>,
            Option<RedstoneCore<WalletUnlocked>>,
            StorkCore<WalletUnlocked>,
        ) {
            let (oracle, pyth, redstone) = setup(REDSTONE_PRECISION, fuel_vm_decimals, true).await;
            let wallet = oracle.wallet();
            let stork = deploy_mock_stork_oracle(&wallet).await;

            // Configure oracle to use Stork
            oracle_abi::set_stork_config(
                &oracle,
                &stork,
                StorkConfig {
                    contract_id: stork.contract_id().into(),
                    feed_id: DEFAULT_STORK_FEED_ID,
                },
            )
            .await
            .unwrap();

            (oracle, pyth, redstone, stork)
        }
    }

    //     #[tokio::test]
    //     async fn test_stork_price_basic() {
    //         let (oracle, _pyth, _redstone, stork) = setup_with_stork(DEFAULT_FUEL_VM_DECIMALS).await;
    //         let price = 1000; // $1000
    //         let timestamp = PYTH_TIMESTAMP;

    //         // Set Stork price
    //         stork_oracle_abi::set_temporal_value(
    //             &stork,
    //             DEFAULT_STORK_FEED_ID,
    //             (price * PRECISION) as u64,
    //             timestamp * NS_TO_SECONDS,
    //         )
    //         .await;

    //         oracle_abi::set_debug_timestamp(&oracle, timestamp).await;
            
    //         let result = oracle_abi::get_price(&oracle, &_pyth, &_redstone).await.value;
    //         assert_eq!(price * PRECISION, result);
    //     }

    //     #[tokio::test]
    //     async fn test_stork_price_stale() {
    //         let (oracle, _pyth, _redstone, stork) = setup_with_stork(DEFAULT_FUEL_VM_DECIMALS).await;
    //         let price = 1000;
    //         let timestamp = PYTH_TIMESTAMP;

    //         // Set stale Stork price
    //         stork_oracle_abi::set_temporal_value(
    //             &stork,
    //             DEFAULT_STORK_FEED_ID,
    //             (price * PRECISION) as u64,
    //             timestamp * NS_TO_SECONDS,
    //         )
    //         .await;

    //         // Set current time to after timeout
    //         oracle_abi::set_debug_timestamp(&oracle, timestamp + ORACLE_TIMEOUT + 1).await;
            
    //         // Should revert with "ORACLE: Price is not fresh"
    //         let result = oracle_abi::get_price(&oracle, &_pyth, &_redstone).await;
    //         assert!(result.transaction_status.unwrap().is_failure());
    //     }

    //     #[tokio::test]
    //     async fn test_stork_price_negative() {
    //         let (oracle, _pyth, _redstone, stork) = setup_with_stork(DEFAULT_FUEL_VM_DECIMALS).await;
    //         let price = -1000;
    //         let timestamp = PYTH_TIMESTAMP;

    //         // Set negative Stork price
    //         stork_oracle_abi::set_temporal_value(
    //             &stork,
    //             DEFAULT_STORK_FEED_ID,
    //             I128::from(price * PRECISION),
    //             timestamp * NS_TO_SECONDS,
    //         )
    //         .await;

    //         oracle_abi::set_debug_timestamp(&oracle, timestamp).await;
            
    //         // Should revert with "ORACLE: Cannot convert negative I128 to u64"
    //         let result = oracle_abi::get_price(&oracle, &_pyth, &_redstone).await;
    //         assert!(result.is_err());
    //     }

    //     #[tokio::test]
    //     async fn test_stork_price_decimal_conversion() {
    //         let fuel_vm_decimals = 8;
    //         let (oracle, _pyth, _redstone, stork) = setup_with_stork(fuel_vm_decimals).await;
    //         let price = 5000; // $5000
    //         let timestamp = PYTH_TIMESTAMP;

    //         // Stork uses 18 decimals internally
    //         stork_oracle_abi::set_temporal_value(
    //             &stork,
    //             DEFAULT_STORK_FEED_ID,
    //             (price * 10u64.pow(18)) as u64,
    //             timestamp * NS_TO_SECONDS,
    //         )
    //         .await;

    //         oracle_abi::set_debug_timestamp(&oracle, timestamp).await;
            
    //         let result = oracle_abi::get_price(&oracle, &_pyth, &_redstone).await.value;
            
    //         // Expected: For 8 decimals, 1_000_000_000 units = 10 units of asset
    //         // So price should be 10 * $5000 = $50000 with 8 decimals
    //         let expected_price = 50000 * 10u64.pow(8);
    //         assert_eq!(expected_price, result);
    //     }

    //     #[tokio::test]
    //     async fn test_stork_price_overflow() {
    //         let (oracle, _pyth, _redstone, stork) = setup_with_stork(DEFAULT_FUEL_VM_DECIMALS).await;
    //         let price = u64::MAX as u128;
    //         let timestamp = PYTH_TIMESTAMP;

    //         // Set maximum possible price
    //         stork_oracle_abi::set_temporal_value(
    //             &stork,
    //             DEFAULT_STORK_FEED_ID,
    //             price,
    //             timestamp * NS_TO_SECONDS,
    //         )
    //         .await;

    //         oracle_abi::set_debug_timestamp(&oracle, timestamp).await;
            
    //         // Should revert with overflow error
    //         let result = oracle_abi::get_price(&oracle, &_pyth, &_redstone).await;
    //         assert!(result.transaction_status.unwrap().is_failure());
    //     }
    // }
}
