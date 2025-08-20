use fuels::types::U256;
use fuels::{prelude::*, types::Identity};
use test_utils::data_structures::ContractInstance;
use test_utils::{
    data_structures::PRECISION,
    interfaces::{
        oracle::{oracle_abi, Oracle, RedstoneConfig, StorkConfig, PythConfig, ORACLE_TIMEOUT},
        pyth_oracle::{
            pyth_oracle_abi, pyth_price_feed_with_time, PythCore, DEFAULT_PYTH_PRICE_ID,
            PYTH_TIMESTAMP,
        },
        redstone_oracle::{redstone_oracle_abi, RedstoneCore, DEFAULT_REDSTONE_PRICE_ID},
        stork_oracle::{stork_oracle_abi, StorkCore, DEFAULT_STORK_FEED_ID, NS_TO_SECONDS},
    },
    setup::common::{deploy_mock_pyth_oracle, deploy_mock_redstone_oracle, deploy_mock_stork_oracle, deploy_oracle},
};

const PYTH_PRECISION: u8 = 12;
const REDSTONE_PRECISION: u32 = 6;
const DEFAULT_FUEL_VM_DECIMALS: u32 = 9;

async fn setup(
    redstone_precision: u32,
    fuel_vm_decimals: u32,
    initialize_stork: bool,
    initialize_pyth: bool,
    initialize_redstone: bool,
) -> (
    ContractInstance<Oracle<Wallet>>,
    Option<StorkCore<Wallet>>,
    Option<PythCore<Wallet>>,
    Option<RedstoneCore<Wallet>>,
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

    // Deploy the oracle contract.
    let oracle = deploy_oracle(
        &wallet,
        fuel_vm_decimals,
        true,
        Identity::Address(wallet.address().into()),
    )
    .await;

    // Do not add the configs from now, add them later.
    let _ = oracle_abi::initialize(
        &oracle,
        None,
        None,
        None,
    )
    .await;

    let mut stork: Option<StorkCore<Wallet>> = None;
    let mut pyth: Option<PythCore<Wallet>> = None;
    let mut redstone: Option<RedstoneCore<Wallet>> = None;

    // If we want to use stork we should deploy a mock and add it to the oracle.
    if initialize_stork {
        stork = Some(deploy_mock_stork_oracle(&wallet).await);
        
        let stork_contract = stork.as_ref().unwrap().clone();

        let _ = oracle_abi::set_stork_config(
            &oracle,
            &stork_contract,
            StorkConfig {
                contract_id: stork_contract.contract_id().into(),
                feed_id: DEFAULT_STORK_FEED_ID,
            },
        )
        .await
        .unwrap();
    }

    // If we want to use pyth we should deploy a mock and add it to the oracle.
    if initialize_pyth {
        pyth = Some(deploy_mock_pyth_oracle(&wallet).await);

        let pyth_contract = pyth.as_ref().unwrap().clone();

        // Set the pyth config for the oracle.
        let _ = oracle_abi::set_pyth_config(&oracle, &pyth_contract, PythConfig {
            contract_id: pyth_contract.contract_id().into(),
            feed_id: DEFAULT_PYTH_PRICE_ID,
            precision: PYTH_PRECISION.into(),
        }).await;
    }

    // If we want to use redstone we should deploy a mock and add it to the oracle.
    if initialize_redstone {
        redstone = Some(deploy_mock_redstone_oracle(&wallet).await);

        let redstone_contract = redstone.as_ref().unwrap().clone();

        let _ = oracle_abi::set_redstone_config(
            &oracle,
            &redstone_contract,
            RedstoneConfig {
                contract_id: redstone_contract.contract_id().into(),
                feed_id: DEFAULT_REDSTONE_PRICE_ID,
                precision: redstone_precision,
            },
        )
        .await
        .unwrap();
    }

    (oracle, stork, pyth, redstone)
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

    // ======================== PYTH (alone and with REDSTONE/STORK) ========================
    mod pyth_only_and_with_redstone {
        use super::*;

        #[tokio::test]
        async fn returns_pyth_price_when_only_pyth_configured() {
            // Ensures Pyth is used when it is the only configured oracle
            let (
                oracle,
                _,
                pyth,
                _,
            ) = setup(
                REDSTONE_PRECISION,
                DEFAULT_FUEL_VM_DECIMALS,
                false,
                true,
                false,
            ).await;

            let pyth_instance = pyth.unwrap();
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                    PYTH_TIMESTAMP,
                    PYTH_PRECISION.into(),
                ),
            )
            .await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                None,
            )
                .await
                .value;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn returns_updated_pyth_price_without_updating_last_price_when_only_pyth() {
            // With only Pyth configured, get_price returns the latest Pyth price, but last_good_price stays at the 
            // freshest publish_time
            let (
                oracle,
                _,
                pyth,
                _,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    false,
                ).await;

            let pyth_instance = pyth.unwrap();
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_two = convert_precision(2 * PRECISION, PYTH_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            
            // Update the price feed for pyth.
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Get the initial last price.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value.price;
            println!("last_price: {:?}", last_price);
            assert_eq!(last_price, 0);

            // Get the price from pyth, which then updates the last price.
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                None,
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            // Check that the last price is updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            println!("last_price: {:?}", last_price);
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                     PYTH_TIMESTAMP - 1,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Get price returns the updated price because no other oracles are configured.
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                None,
            )
                .await
                .value;

            assert_eq!(expected_price_two, price);

            // But the last price was still not updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            println!("last_price: {:?}", last_price);
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);
        }

        #[tokio::test]
        async fn falls_back_to_last_price_when_pyth_stale_and_redstone_stale() {
            // When Pyth is stale and Redstone is configured but stale/older, returns last_good_price
            let (
                oracle,
                _,
                pyth,
                redstone,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            
            // Update the price feed for pyth.
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Get the initial last price.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value.price;
            assert_eq!(last_price, 0);

            // Get the price from pyth, which then updates the last price.
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            // Check that the last price is updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                     PYTH_TIMESTAMP - 1,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Advance time so the newly provided Pyth price is considered stale
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            // Update the redstone prices and timestamp.
            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            )
            .await;

            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP - 1,
            )
            .await;

            // Get price does not return the updated price because redstone is configured.
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            // The last price was not updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);
        }

        #[tokio::test]
        async fn falls_back_to_last_price_when_pyth_stale_and_stork_stale() {
            // When Pyth is stale and Stork is configured but stale/older, returns last_good_price
            let (
                oracle,
                stork,
                pyth,
                _,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    false,
                ).await;

            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            
            // Update the price feed for pyth.
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Update the stork prices (fresh then make stale)
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                PYTH_TIMESTAMP * NS_TO_SECONDS,
            )
            .await;

            // Get the initial last price.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value.price;
            assert_eq!(last_price, 0);

            // Get the price from pyth, which then updates the last price.
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                None,
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            // Check that the last price is updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                     PYTH_TIMESTAMP - 1,
                        PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Advance time so the newly provided Pyth price is considered stale
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            // Update the stork price with a slightly older timestamp to keep it stale
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                3 * 10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                (PYTH_TIMESTAMP - 1) * NS_TO_SECONDS,
            )
            .await;

            // Get price does not return the updated price because stork is configured but stale
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                None,
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            // The last price was not updated.
            let last_price = oracle_abi::get_last_good_price(&oracle).await.unwrap().value;
            assert_eq!(last_price.price, expected_price);
            assert_eq!(last_price.publish_time, PYTH_TIMESTAMP);
        }
    }

    // ======================== PYTH TIMEOUT PATHS (with/without STORK) ========================
    mod pyth_timeout {
        use super::*;

        #[tokio::test]
        async fn uses_redstone_when_pyth_is_stale() {
            // If Pyth is stale and Redstone is fresh, Redstone is used
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone =
                convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price_pyth, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                    PYTH_TIMESTAMP,
                     PYTH_PRECISION.into(),
                ),
            )
            .await;
            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            )
            .await;
            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP + 1,
            )
            .await;
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price_redstone, price);
        }

        #[tokio::test]
        async fn uses_redstone_when_pyth_stale_and_stork_stale() {
            // With Stork stale, behavior matches Pyth/Redstone priority rules
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone =
                convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            // Make stork stale so it doesn't affect selection
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                (PYTH_TIMESTAMP - ORACLE_TIMEOUT - 1) * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            ).await;

            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_pyth, price);

            // Advance and make redstone the live source
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;
            redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP + 1).await;

            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_redstone, price);
        }

        #[tokio::test]
        async fn uses_stork_when_fresh_over_pyth_and_redstone() {
            // Stork takes priority when fresh, even if Pyth and Redstone are available
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            // Set fresh Stork value equal to expected pyth-adjusted price
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                PYTH_TIMESTAMP * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            ).await;
            redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP).await;

            // Stork should take priority
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_pyth, price);

            // Even after Pyth becomes stale and Redstone is fresh, if Stork is still fresh, Stork wins
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + 1).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP + 1).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_pyth, price);
        }

        mod redstone_timeout {
            use super::*;

            mod pyth_timestamp_more_recent {
                use super::*;

                #[tokio::test]
                async fn uses_pyth_when_it_is_more_recent_after_timeout() {
                    // After timeout but with newer Pyth publish_time, prefer Pyth over Redstone
                    let (
                        oracle,
                        _,
                        pyth,
                        redstone_wrapped,
                    ) =
                        setup(
                            REDSTONE_PRECISION,
                            DEFAULT_FUEL_VM_DECIMALS,
                            false,
                            true,
                            true,
                        ).await;


                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

                    let pyth_instance = pyth.unwrap();
                    let redstone_instance = redstone_wrapped.unwrap();

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth_instance,
                        pyth_price_feed_with_time(
                            1,
                             PYTH_TIMESTAMP,
                              PYTH_PRECISION.into(),
                        ),
                    )
                    .await;

                    let price = oracle_abi::get_price(
                        &oracle,
                        None,
                        Some(&pyth_instance),
                        Some(&redstone_instance),
                    )
                    .await
                    .value;

                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                        .await;

                    pyth_oracle_abi::update_price_feeds(
                        &pyth_instance,
                        pyth_price_feed_with_time(
                            2,
                            PYTH_TIMESTAMP + 1,
                            PYTH_PRECISION.into(),
                        ),
                    )
                    .await;

                    redstone_oracle_abi::write_prices(
                        &redstone_instance,
                        redstone_feed(3),
                    ).await;
                    redstone_oracle_abi::set_timestamp(
                        &redstone_instance,
                        PYTH_TIMESTAMP,
                    ).await;

                    let price = oracle_abi::get_price(
                        &oracle,
                        None,
                        Some(&pyth_instance),
                        Some(&redstone_instance),
                    )
                    .await
                    .value;

                    assert_eq!(expected_price * 2, price);
                }

                #[tokio::test]
                async fn uses_pyth_then_redstone_with_stork_stale() {
                    // With Stork stale, prefer fresh Pyth; then after timeout prefer fresh Redstone
                    let (
                        oracle,
                        stork,
                        pyth,
                        redstone_wrapped,
                    ) =
                        setup(
                            REDSTONE_PRECISION,
                            DEFAULT_FUEL_VM_DECIMALS,
                            true,
                            true,
                            true,
                        ).await;

                    let stork_instance = stork.unwrap();
                    let pyth_instance = pyth.unwrap();
                    let redstone_instance = redstone_wrapped.unwrap();

                    let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

                    // Stork stale setup
                    stork_oracle_abi::set_temporal_value(
                        &stork_instance,
                        DEFAULT_STORK_FEED_ID,
                        10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                        (PYTH_TIMESTAMP - ORACLE_TIMEOUT - 1) * NS_TO_SECONDS,
                    ).await;

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth_instance,
                        pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
                    ).await;

                    let price = oracle_abi::get_price(
                        &oracle,
                        Some(&stork_instance),
                        Some(&pyth_instance),
                        Some(&redstone_instance),
                    ).await.value;
                    assert_eq!(expected_price, price);

                    oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2).await;
                    pyth_oracle_abi::update_price_feeds(
                        &pyth_instance,
                        pyth_price_feed_with_time(2, PYTH_TIMESTAMP + 1, PYTH_PRECISION.into()),
                    ).await;
                    redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
                    redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP).await;

                    let price = oracle_abi::get_price(
                        &oracle,
                        Some(&stork_instance),
                        Some(&pyth_instance),
                        Some(&redstone_instance),
                    ).await.value;
                    assert_eq!(expected_price * 2, price);
                }
            }
        }

        #[tokio::test]
        async fn falls_back_to_last_price_when_both_oracles_unusable() {
            // When Pyth is stale and Redstone unusable, returns last_good_price
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1)
                .await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP).await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn falls_back_to_last_price_with_stork_stale_when_others_unusable() {
            // With Stork stale and other oracles unusable, returns last_good_price
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            // Stork stale
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                (PYTH_TIMESTAMP - ORACLE_TIMEOUT - 1) * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                        PYTH_TIMESTAMP,
                        PYTH_PRECISION.into(),
                ),
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;

            assert_eq!(expected_price, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 1).await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                        PYTH_TIMESTAMP,
                        PYTH_PRECISION.into(),
                ),
            ).await;

            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            ).await;
            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP,
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;

            assert_eq!(expected_price, price);
        }
    }

    // ======================== REDSTONE MORE RECENT PATHS (with/without STORK) ========================
    mod redstone_more_recent_scenarios {
        use super::*;

        #[tokio::test]
        async fn uses_redstone_when_redstone_more_recent() {
            // If Redstone is more recent than Pyth (after Pyth timeout), use Redstone
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let expected_price_pyth =
                convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone =
                convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                        PYTH_TIMESTAMP,
                        PYTH_PRECISION.into(),
                ),
            )
            .await;
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price_pyth, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                .await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(2, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            )
            .await;

            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            ).await;
            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP + 1,
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            let redstone_price = redstone_oracle_abi::read_prices(
                &redstone_instance,
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
        async fn uses_redstone_when_more_recent_with_stork_stale() {
            // With Stork stale, prefer Redstone when more recent than Pyth
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());
            let expected_price_redstone = convert_precision(3 * PRECISION, REDSTONE_PRECISION.into());

            // Stork stale
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                (PYTH_TIMESTAMP - ORACLE_TIMEOUT - 1) * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_pyth, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2).await;
            redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP + 1).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            assert_eq!(expected_price_redstone, price);
        }

        #[tokio::test]
        async fn uses_stork_when_fresh_over_redstone() {
            // Stork takes priority over Redstone when Stork is fresh
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let expected_price_pyth = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            // Fresh Stork value
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                PYTH_TIMESTAMP * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(1, PYTH_TIMESTAMP, PYTH_PRECISION.into()),
            ).await;
            redstone_oracle_abi::write_prices(&redstone_instance, redstone_feed(3)).await;
            redstone_oracle_abi::set_timestamp(&redstone_instance, PYTH_TIMESTAMP + 1).await;

            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;
            // Stork still takes priority
            assert_eq!(expected_price_pyth, price);
        }

        #[tokio::test]
        async fn falls_back_to_last_price() {
            // If Pyth is stale and Redstone is not usable/newer, return last_good_price
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                .await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                     PYTH_TIMESTAMP,
                      PYTH_PRECISION.into(),
                ),
            )
            .await;

            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            ).await;
            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP,
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
                .await
                .value;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn falls_back_to_last_price_with_stork_stale() {
            // With Stork stale and both Pyth/Redstone unusable, return last_good_price
            let (
                oracle,
                stork,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    true,
                    true,
                    true,
                ).await;
            let expected_price = convert_precision(1 * PRECISION, PYTH_PRECISION.into());

            let stork_instance = stork.unwrap();
            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            // Stork stale
            stork_oracle_abi::set_temporal_value(
                &stork_instance,
                DEFAULT_STORK_FEED_ID,
                10u64.pow((27 - PYTH_PRECISION as u32) as u32),
                (PYTH_TIMESTAMP - ORACLE_TIMEOUT - 1) * NS_TO_SECONDS,
            ).await;

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    1,
                        PYTH_TIMESTAMP,
                        PYTH_PRECISION.into(),
                ),
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;

            assert_eq!(expected_price, price);

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + ORACLE_TIMEOUT + 2)
                .await;

            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(
                    2,
                        PYTH_TIMESTAMP,
                        PYTH_PRECISION.into(),
                ),
            ).await;

            redstone_oracle_abi::write_prices(
                &redstone_instance,
                redstone_feed(3),
            ).await;
            redstone_oracle_abi::set_timestamp(
                &redstone_instance,
                PYTH_TIMESTAMP,
            ).await;
            let price = oracle_abi::get_price(
                &oracle,
                Some(&stork_instance),
                Some(&pyth_instance),
                Some(&redstone_instance),
            ).await.value;

            assert_eq!(expected_price, price);
        }
    }

    // ======================== PYTH CONFIDENCE BEHAVIOR ========================
    mod confidence_check {
        use test_utils::interfaces::pyth_oracle::pyth_price_feed_with_confidence;

        use super::*;

        #[tokio::test]
        async fn ignores_confidence_when_within_threshold_if_pyth_fresh() {
            // Confidence is adjusted but does not affect selection when Pyth is fresh
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let price = 1 * PRECISION;
            let confidence = price / 100; // 1%

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_confidence(
                    price,
                    PYTH_TIMESTAMP,
                    confidence,
                    PYTH_PRECISION.into(),
                ),
            )
            .await;

            let result = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;

            assert_eq!(convert_precision(price, PYTH_PRECISION.into()), result);
        }

        #[tokio::test]
        async fn ignores_confidence_even_when_outside_threshold_if_pyth_fresh() {
            // Even if Pyth confidence is high, fresh Pyth is still used
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            let pyth_price = 1 * PRECISION;
            let pyth_confidence = pyth_price / 20; // 5%

            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_confidence(
                    pyth_price,
                    PYTH_TIMESTAMP,
                    pyth_confidence,
                    PYTH_PRECISION.into(),
                ),
            )
            .await;

            // Current implementation ignores confidence thresholds and uses fresh Pyth
            let result = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;

            assert_eq!(convert_precision(pyth_price, PYTH_PRECISION.into()), result);
        }
    }

    // ======================== FUEL VM DECIMALS ADJUSTMENTS ========================
    mod fuel_vm_decimals {
        use super::*;

        #[tokio::test]
        async fn adjusts_price_for_fuel_vm_decimals_8() {
            // Verifies price scaling when Fuel VM uses 8 decimals
            let fuel_vm_decimals = 8;
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    fuel_vm_decimals,
                    false,
                    true,
                    true,
                ).await;

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            // Set a price of $5000 for 1 unit of the asset
            let pyth_price = 5000;
            let pyth_timestamp = PYTH_TIMESTAMP;

            oracle_abi::set_debug_timestamp(&oracle, pyth_timestamp).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(pyth_price, pyth_timestamp, 9),
            )
            .await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;

            // Expected price calculation:
            // 1. Convert Pyth price to 8 decimal precision: 5000 * 10^8 = 500_000_000_000
            // 2. Multiply by 10 because 1_000_000_000 units with 8 decimals is 10 units of the asset
            let expected_price = 5000 * PRECISION * 10;

            assert_eq!(expected_price, price);
        }

        #[tokio::test]
        async fn adjusts_price_for_fuel_vm_decimals_12() {
            // Verifies price scaling when Fuel VM uses 12 decimals
            let fuel_vm_decimals = 12;
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    fuel_vm_decimals,
                    false,
                    true,
                    true,
                ).await;

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            // Set a price of $5000 for 1 unit of the asset
            let pyth_price = 5000;
            let pyth_timestamp = PYTH_TIMESTAMP;

            oracle_abi::set_debug_timestamp(&oracle, pyth_timestamp).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_time(pyth_price, pyth_timestamp, 9),
            )
            .await;

            let price = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;

            // Expected price calculation:
            // 1. Convert Pyth price to 12 decimal precision: 5000 * 10^12 = 5_000_000_000_000_000
            // 2. Divide by 1000 because 1_000_000_000 units with 12 decimals is 0.001 units of the asset
            let expected_price = 5000 * PRECISION / 1000;

            assert_eq!(expected_price, price);
        }
    }

    // ======================== PYTH EXPONENT NORMALIZATION ========================
    mod pyth_exponent_changes {
        use test_utils::interfaces::pyth_oracle::pyth_price_feed_with_confidence;

        use super::*;

        #[tokio::test]
        async fn pyth_price_adjustment_is_consistent_across_exponents() {
            // Pyth price normalization is consistent across different reported exponents
            let (
                oracle,
                _,
                pyth,
                redstone_wrapped,
            ) =
                setup(
                    REDSTONE_PRECISION,
                    DEFAULT_FUEL_VM_DECIMALS,
                    false,
                    true,
                    true,
                ).await;
            let expected_price = 1 * PRECISION; // $1.00 with 9 decimal places

            let pyth_instance = pyth.unwrap();
            let redstone_instance = redstone_wrapped.unwrap();

            // Test with exponent 9
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_confidence(1_000_000_000, PYTH_TIMESTAMP, 0, 9),
            )
            .await;
            let price_exp_9 = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;
            assert_eq!(expected_price, price_exp_9);

            // Test with exponent 6
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + 1).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_confidence(1_000_000, PYTH_TIMESTAMP + 1, 0, 6),
            )
            .await;
            let price_exp_6 = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;
            assert_eq!(expected_price, price_exp_6);

            // Test with exponent 12
            oracle_abi::set_debug_timestamp(&oracle, PYTH_TIMESTAMP + 2).await;
            pyth_oracle_abi::update_price_feeds(
                &pyth_instance,
                pyth_price_feed_with_confidence(1_000_000_000_000, PYTH_TIMESTAMP + 2, 0, 12),
            )
            .await;
            let price_exp_12 = oracle_abi::get_price(
                &oracle,
                None,
                Some(&pyth_instance),
                Some(&redstone_instance),
            )
            .await
            .value;
            assert_eq!(expected_price, price_exp_12);

            // Assert that all prices are equal
            assert_eq!(price_exp_9, price_exp_6);
            assert_eq!(price_exp_9, price_exp_12);
            assert_eq!(price_exp_6, price_exp_12);
        }
    }
}
