pub mod utils {
    use csv::ReaderBuilder;
    use fuels::accounts::signers::private_key::PrivateKeySigner;
    use fuels::accounts::{provider::Provider, wallet::Wallet};
    use fuels::crypto::SecretKey;
    use fuels::prelude::*;
    use fuels::types::bech32::Bech32ContractId;
    use fuels::types::{Bits256, Identity, U256};
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use test_utils::data_structures::{
        AssetContracts, AssetContractsOptionalOracles, ContractInstance, PRECISION,
    };
    use test_utils::interfaces::oracle::oracle_abi;
    use test_utils::interfaces::pyth_oracle::pyth_oracle_abi;
    use test_utils::interfaces::redstone_oracle::redstone_oracle_abi;
    use test_utils::interfaces::stork_oracle::{stork_oracle_abi, StorkCore, I128};
    use test_utils::interfaces::vesting::{VestingSchedule, TOTAL_AMOUNT_VESTED};
    use test_utils::setup::common::get_absolute_path_from_relative;
    use test_utils::{
        data_structures::ProtocolContracts,
        interfaces::{
            active_pool::ActivePool,
            borrow_operations::BorrowOperations,
            coll_surplus_pool::CollSurplusPool,
            community_issuance::CommunityIssuance,
            default_pool::DefaultPool,
            fpt_staking::FPTStaking,
            fpt_token::FPTToken,
            oracle::{PythConfig, RedstoneConfig, StorkConfig},
            protocol_manager::ProtocolManager,
            pyth_oracle::PythCore,
            redstone_oracle::RedstoneCore,
            sorted_troves::SortedTroves,
            stability_pool::StabilityPool,
            token::Token,
            trove_manager::TroveManagerContract,
            vesting::VestingContract,
        },
    };

    use crate::constants::{
        MAINNET_CONTRACTS_FILE, MAINNET_RPC, TESTNET_CONTRACTS_FILE, TESTNET_RPC,
    };
    pub async fn setup_wallet() -> Wallet {
        let network =
            std::env::var("NETWORK").expect("NETWORK must be set to 'mainnet' or 'testnet'");

        let rpc: String = match network.as_str() {
            "mainnet" => MAINNET_RPC.to_string(),
            "testnet" => TESTNET_RPC.to_string(),
            _ => panic!("❌ NETWORK must be 'mainnet' or 'testnet'"),
        };

        let provider = match Provider::connect(rpc).await {
            Ok(p) => p,
            Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
        };

        let secret = match std::env::var("SECRET") {
            Ok(s) => s,
            Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
        };

        let secret_key =
            SecretKey::new_from_mnemonic_phrase_with_path(&secret, "m/44'/1179993420'/0'/0/0")
                .unwrap();

        Wallet::new(PrivateKeySigner::new(secret_key), provider)
    }

    pub fn load_core_contracts(wallet: Wallet, is_testnet: bool) -> ProtocolContracts<Wallet> {
        let json = std::fs::read_to_string(match is_testnet {
            true => TESTNET_CONTRACTS_FILE,
            false => MAINNET_CONTRACTS_FILE,
        })
        .unwrap();
        let contracts: serde_json::Value = serde_json::from_str(&json).unwrap();

        let borrow_operations_contract_id: Bech32ContractId = contracts["borrow_operations"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let borrow_operations_implementation_id: ContractId = contracts
            ["borrow_operations_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let borrow_operations = ContractInstance::new(
            BorrowOperations::new(borrow_operations_contract_id.clone(), wallet.clone()),
            borrow_operations_implementation_id.into(),
        );
        let usdm_contract_id: Bech32ContractId =
            contracts["usdm"].as_str().unwrap().parse().unwrap();
        let usdm_implementation_id: ContractId = contracts["usdm_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let usdm = ContractInstance::new(
            test_utils::interfaces::usdm_token::USDMToken::new(usdm_contract_id, wallet.clone()),
            usdm_implementation_id.into(),
        );

        let stability_pool_contract_id: Bech32ContractId = contracts["stability_pool"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let stability_pool_implementation_id: ContractId = contracts
            ["stability_pool_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let stability_pool = ContractInstance::new(
            StabilityPool::new(stability_pool_contract_id.clone(), wallet.clone()),
            stability_pool_implementation_id.into(),
        );

        let protocol_manager_contract_id: Bech32ContractId = contracts["protocol_manager"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let protocol_manager_implementation_id: ContractId = contracts
            ["protocol_manager_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let protocol_manager = ContractInstance::new(
            ProtocolManager::new(protocol_manager_contract_id.clone(), wallet.clone()),
            protocol_manager_implementation_id.into(),
        );

        let fpt_staking_contract_id: Bech32ContractId =
            contracts["fpt_staking"].as_str().unwrap().parse().unwrap();
        let fpt_staking_implementation_id: ContractId = contracts["fpt_staking_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let fpt_staking = ContractInstance::new(
            FPTStaking::new(fpt_staking_contract_id.clone(), wallet.clone()),
            fpt_staking_implementation_id.into(),
        );

        let fpt_token_contract_id: Bech32ContractId =
            contracts["fpt_token"].as_str().unwrap().parse().unwrap();
        let fpt_token_implementation_id: ContractId = contracts["fpt_token_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let fpt_token = ContractInstance::new(
            FPTToken::new(fpt_token_contract_id.clone(), wallet.clone()),
            fpt_token_implementation_id.into(),
        );

        let community_issuance_contract_id: Bech32ContractId = contracts["community_issuance"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let community_issuance_implementation_id: ContractId = contracts
            ["community_issuance_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let community_issuance = ContractInstance::new(
            CommunityIssuance::new(community_issuance_contract_id.clone(), wallet.clone()),
            community_issuance_implementation_id.into(),
        );

        let coll_surplus_pool_contract_id: Bech32ContractId = contracts["coll_surplus_pool"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let coll_surplus_pool_implementation_id: ContractId = contracts
            ["coll_surplus_pool_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let coll_surplus_pool = ContractInstance::new(
            CollSurplusPool::new(coll_surplus_pool_contract_id.clone(), wallet.clone()),
            coll_surplus_pool_implementation_id.into(),
        );

        let default_pool_contract_id: Bech32ContractId =
            contracts["default_pool"].as_str().unwrap().parse().unwrap();
        let default_pool_implementation_id: ContractId = contracts
            ["default_pool_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let default_pool = ContractInstance::new(
            DefaultPool::new(default_pool_contract_id.clone(), wallet.clone()),
            default_pool_implementation_id.into(),
        );

        let active_pool_contract_id: Bech32ContractId =
            contracts["active_pool"].as_str().unwrap().parse().unwrap();
        let active_pool_implementation_id: ContractId = contracts["active_pool_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let active_pool = ContractInstance::new(
            ActivePool::new(active_pool_contract_id.clone(), wallet.clone()),
            active_pool_implementation_id.into(),
        );

        let sorted_troves_contract_id: Bech32ContractId = contracts["sorted_troves"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let sorted_troves_implementation_id: ContractId = contracts
            ["sorted_troves_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let sorted_troves = ContractInstance::new(
            SortedTroves::new(sorted_troves_contract_id.clone(), wallet.clone()),
            sorted_troves_implementation_id.into(),
        );

        let vesting_contract_id: Bech32ContractId = contracts["vesting_contract"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let vesting_contract_implementation_id: ContractId = contracts
            ["vesting_contract_implementation_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let vesting_contract = ContractInstance::new(
            VestingContract::new(vesting_contract_id.clone(), wallet.clone()),
            vesting_contract_implementation_id.into(),
        );

        let fpt_asset_id: AssetId =
            AssetId::from_str(contracts["fpt_asset_id"].as_str().unwrap()).unwrap();

        let usdm_asset_id: AssetId =
            AssetId::from_str(contracts["usdm_asset_id"].as_str().unwrap()).unwrap();

        let asset_contracts = contracts["asset_contracts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|asset_contract| {
                let asset_contract_id: Bech32ContractId = asset_contract["asset_contract"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap();
                let asset_id =
                    AssetId::from_str(asset_contract["asset_id"].as_str().unwrap()).unwrap();
                let oracle_contract_id: Bech32ContractId =
                    asset_contract["oracle"].as_str().unwrap().parse().unwrap();
                let oracle_implementation_id: ContractId = asset_contract
                    ["oracle_implementation_id"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap();
                let trove_manager_contract_id: Bech32ContractId = asset_contract["trove_manager"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap();
                let trove_manager_implementation_id: ContractId = asset_contract
                    ["trove_manager_implementation_id"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap();
                let stork_contract_id: Bech32ContractId = asset_contract["stork_contract"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(Bech32ContractId::default());
                let pyth_contract_id: Bech32ContractId = asset_contract["pyth_contract"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(Bech32ContractId::default());
                let redstone_contract_id: Bech32ContractId = asset_contract["redstone_contract"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(Bech32ContractId::default());

                AssetContracts {
                    asset: Token::new(asset_contract_id, wallet.clone()),
                    asset_id,
                    oracle: ContractInstance::new(
                        test_utils::interfaces::oracle::Oracle::new(
                            oracle_contract_id.clone(),
                            wallet.clone(),
                        ),
                        oracle_implementation_id.into(),
                    ),
                    trove_manager: ContractInstance::new(
                        TroveManagerContract::new(
                            trove_manager_contract_id.clone(),
                            wallet.clone(),
                        ),
                        trove_manager_implementation_id.into(),
                    ),
                    fuel_vm_decimals: asset_contract["fuel_vm_decimals"].as_u64().unwrap() as u32,
                    mock_stork_oracle: StorkCore::new(stork_contract_id, wallet.clone()),
                    mock_pyth_oracle: PythCore::new(pyth_contract_id, wallet.clone()),
                    mock_redstone_oracle: RedstoneCore::new(redstone_contract_id, wallet.clone()),
                    stork_feed_id: Bits256::from_hex_str(
                        asset_contract["stork_feed_id"].as_str().unwrap_or(""),
                    )
                    .unwrap_or(Bits256::zeroed()),
                    pyth_price_id: Bits256::from_hex_str(
                        asset_contract["pyth_price_id"].as_str().unwrap_or(""),
                    )
                    .unwrap_or(Bits256::zeroed()),
                    redstone_precision: asset_contract["redstone_precision"].as_u64().unwrap_or(9)
                        as u32,
                    redstone_price_id: U256::from_str(
                        asset_contract["redstone_price_id"].as_str().unwrap_or("0"),
                    )
                    .unwrap(),
                }
            })
            .collect();

        let protocol_contracts = ProtocolContracts {
            borrow_operations,
            usdm,
            stability_pool,
            protocol_manager,
            asset_contracts,
            fpt_staking,
            fpt_token,
            fpt_asset_id,
            usdm_asset_id,
            community_issuance,
            vesting_contract,
            coll_surplus_pool,
            sorted_troves,
            default_pool,
            active_pool,
        };

        protocol_contracts
    }

    pub async fn is_testnet(wallet: Wallet) -> bool {
        let network_name = wallet.provider().chain_info().await.unwrap().name;
        network_name.to_lowercase().contains("testnet")
    }

    pub fn write_asset_contracts_to_file(
        asset_contracts: Vec<AssetContractsOptionalOracles<Wallet>>,
        is_testnet: bool,
    ) {
        // Read existing contracts.json
        let mut contracts: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(match is_testnet {
                true => TESTNET_CONTRACTS_FILE,
                false => MAINNET_CONTRACTS_FILE,
            })
            .expect("Failed to read contracts.json"),
        )
        .expect("Failed to parse contracts.json");

        // Get existing asset_contracts or create an empty array if it doesn't exist
        let mut existing_asset_contracts = contracts["asset_contracts"]
            .as_array()
            .cloned()
            .unwrap_or_else(Vec::new);

        // Add new asset_contracts to the existing ones
        for asset_contract in asset_contracts {
            existing_asset_contracts.push(json!({
                "symbol": asset_contract.symbol,
                "oracle": asset_contract.oracle.contract.contract_id().to_string(),
                "oracle_implementation_id": format!("0x{}", asset_contract.oracle.implementation_id.to_string()),
                "trove_manager": asset_contract.trove_manager.contract.contract_id().to_string(),
                "trove_manager_implementation_id": format!("0x{}", asset_contract.trove_manager.implementation_id.to_string()),
                "asset_contract": asset_contract.asset.contract_id().to_string(),
                "asset_id": format!("0x{}", asset_contract.asset_id.to_string()),
                "pyth_price_id": match &asset_contract.pyth_price_id {
                    Some(pyth_price_id) => json!(to_hex_str(pyth_price_id)),
                    None => json!(null),
                },
                "pyth_contract": match &asset_contract.mock_pyth_oracle {
                    Some(pyth_contract) => json!(pyth_contract.contract_id().to_string()),
                    None => json!(null),
                },
                "stork_feed_id": match &asset_contract.stork_feed_id {
                    Some(stork_feed_id) => json!(to_hex_str(stork_feed_id)),
                    None => json!(null),
                },
                "stork_contract": match &asset_contract.mock_stork_oracle {
                    Some(stork_contract) => json!(stork_contract.contract_id().to_string()),
                    None => json!(null),
                },
                "redstone": match &asset_contract.redstone_config {
                    Some(redstone_config) => {
                        json!({
                            "redstone_contract": redstone_config.contract.to_string(),
                            "redstone_price_id": redstone_config.price_id.to_string(),
                            "redstone_precision": redstone_config.precision,
                        })
                    },
                    None => json!(null),
                },
                "fuel_vm_decimals": asset_contract.fuel_vm_decimals,
            }));
        }

        // Update asset_contracts field with the combined list
        contracts["asset_contracts"] = json!(existing_asset_contracts);

        // Write updated contracts back to file
        let mut file = File::create(match is_testnet {
            true => TESTNET_CONTRACTS_FILE,
            false => MAINNET_CONTRACTS_FILE,
        })
        .expect("Failed to open contracts.json for writing");
        file.write_all(serde_json::to_string_pretty(&contracts).unwrap().as_bytes())
            .expect("Failed to write to contracts.json");
    }

    pub async fn query_oracles(
        asset_contracts: &AssetContractsOptionalOracles<Wallet>,
        wallet: Wallet,
    ) {
        // PYTH
        let (pyth_contract, pyth_config) = match &asset_contracts.mock_pyth_oracle {
            Some(oracle) => {
                let pyth_config = PythConfig {
                    contract_id: oracle.contract_id().into(),
                    feed_id: asset_contracts.pyth_price_id.unwrap(),
                    precision: 8, // Pyth uses 8 decimals by default
                };

                let current_pyth_price =
                    pyth_oracle_abi::price_unsafe(&oracle, &asset_contracts.pyth_price_id.unwrap())
                        .await
                        .value;

                let pyth_precision = current_pyth_price.exponent as usize;

                println!(
                    "Current pyth price: {:.precision$}",
                    current_pyth_price.price as f64
                        / 10f64.powi(pyth_precision.try_into().unwrap()),
                    precision = pyth_precision
                );

                (Some(oracle), Some(pyth_config))
            }
            None => {
                println!("No pyth config found");
                (None, None)
            }
        };

        // STORK
        let (stork_contract, stork_config) = match &asset_contracts.mock_stork_oracle {
            Some(oracle) => {
                let stork_config = StorkConfig {
                    contract_id: oracle.contract_id().into(),
                    feed_id: asset_contracts.stork_feed_id.unwrap(),
                };

                let stork_value = stork_oracle_abi::get_temporal_value(
                    &oracle,
                    asset_contracts.stork_feed_id.unwrap(),
                )
                .await
                .quantized_value;

                // Convert stork oracle value to fuel u64 representation
                let stork_price = stork_i128_to_u64(&stork_value)
                    .expect("Failed to convert stork oracle value to fuel u64");

                println!("Stork oracle price: {}", stork_price);

                (Some(oracle), Some(stork_config))
            }
            None => {
                println!("No stork config found");
                (None, None)
            }
        };

        // REDSTONE
        let (redstone_contract, redstone_config) = match &asset_contracts.redstone_config {
            Some(config) => {
                let oracle = RedstoneCore::new(config.contract, wallet.clone());
                let config = RedstoneConfig {
                    contract_id: config.contract,
                    feed_id: config.price_id,
                    precision: config.precision,
                };

                // Get current price
                let current_redstone_price =
                    redstone_oracle_abi::read_prices(&oracle, vec![config.feed_id])
                        .await
                        .value[0]
                        .as_u64();

                let redstone_precision = config.precision as usize;

                println!(
                    "Current redstone price: {:.precision$}",
                    current_redstone_price as f64
                        / 10f64.powi(redstone_precision.try_into().unwrap()),
                    precision = redstone_precision
                );

                (Some(oracle), Some(config))
            }
            None => {
                println!("No redstone config found");
                (None, None)
            }
        };

        // Initialize oracle with all available price feeds
        if let (Some(stork_contract), Some(stork_config)) = (stork_contract, stork_config) {
            oracle_abi::set_stork_config(&asset_contracts.oracle, stork_contract, stork_config)
                .await;
        }
        if let (Some(pyth_contract), Some(pyth_config)) = (pyth_contract, pyth_config) {
            oracle_abi::set_pyth_config(&asset_contracts.oracle, pyth_contract, pyth_config).await;
        }
        if let (Some(redstone_contract), Some(redstone_config)) =
            (redstone_contract.clone(), redstone_config)
        {
            oracle_abi::set_redstone_config(
                &asset_contracts.oracle,
                &redstone_contract,
                redstone_config,
            )
            .await
            .unwrap();
        }

        let current_price = oracle_abi::get_price(&asset_contracts.oracle).await;

        println!(
            "Current oracle proxy price: {:.9}",
            current_price as f64 / 1_000_000_000.0
        );
    }

    pub fn to_hex_str(bits: &Bits256) -> String {
        format!("0x{}", hex::encode(bits.0))
    }

    /// Converts a stork oracle underlying value to fuel u64 representation
    /// This implements the same logic as the oracle contract's _stork_underlying_to_fuel_u64 function
    /// but optimized for Rust and with better error handling
    pub fn stork_underlying_to_fuel_u64(
        underlying: u128,
        fuel_decimal_representation: u32,
    ) -> Result<u64> {
        const BIAS: u128 = 1u128 << 127; // 2^127
        const U64_MAX: u128 = u64::MAX as u128;
        const DECIMALS_18: u32 = 18;

        // If the underlying is below the bias, the logical signed value is negative
        if underlying < BIAS {
            return Err(fuels::types::errors::Error::Other(
                "Negative value not allowed".to_string(),
            ));
        }

        let magnitude = underlying - BIAS;

        // Apply decimal conversion from 18 to fuel_decimal_representation with rounding up on downscale
        let adjusted_value = if fuel_decimal_representation < DECIMALS_18 {
            let precision = DECIMALS_18 - fuel_decimal_representation;
            let divisor = 10u128.pow(precision);

            // Round up on downscale: (magnitude + divisor - 1) / divisor
            (magnitude + divisor - 1) / divisor
        } else if fuel_decimal_representation > DECIMALS_18 {
            let decimal_diff = fuel_decimal_representation - DECIMALS_18;
            let multiplier = 10u128.pow(decimal_diff);

            // Check if multiplication would exceed u64 maximum
            if magnitude > U64_MAX / multiplier {
                return Err(fuels::types::errors::Error::Other(
                    "Multiplication would exceed u64 maximum".to_string(),
                ));
            }

            magnitude * multiplier
        } else {
            magnitude
        };

        // Bound to u64
        if adjusted_value > U64_MAX {
            return Err(fuels::types::errors::Error::Other(
                "Price value exceeds u64 maximum".to_string(),
            ));
        }

        Ok(adjusted_value as u64)
    }

    /// Helper function to convert stork oracle I128 to fuel u64 with default fuel decimals (9)
    pub fn stork_i128_to_u64(i128_value: &I128) -> Result<u64> {
        let fuel_decimals: u32 = 9;
        stork_underlying_to_fuel_u64(i128_value.underlying, fuel_decimals)
    }

    pub fn load_vesting_schedules_from_csv(
        path: &str,
        cliff_percentage: f64,
        seconds_to_cliff: u64,
        seconds_vesting_duration: u64,
        treasury_identity: Identity,
    ) -> Vec<VestingSchedule> {
        let absolute_path = get_absolute_path_from_relative(path);
        let file = File::open(&absolute_path).expect("Failed to open file");
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .has_headers(false)
            .from_reader(file);

        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let cliff_timestamp = now_unix + seconds_to_cliff;
        let end_timestamp = cliff_timestamp + seconds_vesting_duration;
        let now_unix_and_5_minutes = now_unix + 5 * 60;

        let now_and_5_minutes = tai64::Tai64::from_unix(now_unix_and_5_minutes.try_into().unwrap());
        let cliff_timestamp = tai64::Tai64::from_unix(cliff_timestamp.try_into().unwrap());
        let end_timestamp = tai64::Tai64::from_unix(end_timestamp.try_into().unwrap());

        let mut schedules = Vec::new();

        for result in reader.records() {
            let record = result.expect("Failed to read CSV record");
            if record.len() < 5 || record[1].is_empty() {
                panic!("Invalid row found in CSV: {:?}", record);
            }

            // println!("record: {:?}", record);

            let total_amount = (record[1].replace([',', '"'], "").parse::<f64>().unwrap()
                * PRECISION as f64) as u64;
            let recipient = if !record[2].is_empty() {
                Identity::Address(Address::from_str(&record[2]).unwrap())
            } else if !record[3].is_empty() {
                panic!("ETH addresses are not supported yet: {:?}", record);
            } else {
                panic!("No valid wallet address found in row: {:?}", record);
            };

            let schedule = VestingSchedule {
                cliff_amount: (total_amount as f64 * cliff_percentage) as u64,
                cliff_timestamp: cliff_timestamp.0,
                end_timestamp: end_timestamp.0,
                claimed_amount: 0,
                total_amount,
                recipient,
            };

            schedules.push(schedule);
        }
        // take the sum of all total_amounts
        let total_sum: u64 = schedules.iter().map(|s| s.total_amount).sum();
        println!("Total sum of all vesting amounts: {}", total_sum);
        // add one more schedule with the remaining amount
        let remaining_amount = TOTAL_AMOUNT_VESTED - total_sum;
        println!("Remaining amount: {}", remaining_amount);
        // treasury vesting schedule
        schedules.push(VestingSchedule {
            cliff_amount: remaining_amount,
            cliff_timestamp: now_and_5_minutes.0, // cliff timestamp is now + 5 minutes
            end_timestamp: end_timestamp.0,
            claimed_amount: 0,
            total_amount: remaining_amount,
            recipient: treasury_identity,
        });
        schedules
    }
}
