use fuels::{
    prelude::*,
    types::{Bits256, ContractId},
};

// We'll use the generated types from the ABI
abigen!(Contract(
    name = "StorkCore",
    abi = "contracts/mock-stork-contract/out/debug/mock-stork-contract-abi.json"
));

pub const DEFAULT_STORK_FEED_ID: Bits256 = Bits256([0; 32]);
pub const NS_TO_SECONDS: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub struct StorkConfig {
    pub contract_id: ContractId,
    pub feed_id: Bits256,
}

pub mod stork_oracle_abi {
    use super::*;

    pub async fn set_temporal_value(
        contract: &StorkCore<WalletUnlocked>,
        feed_id: Bits256,
        value: u64,
        timestamp_ns: u64,
    ) {
        let temporal_value = TemporalNumericValue {
            timestamp_ns,
            quantized_value: I128 { 
                underlying: value as u128,
            },
        };

        let input = TemporalNumericValueInput {
            temporal_numeric_value: temporal_value,
            id: feed_id,
            publisher_merkle_root: Bits256([0; 32]),
            value_compute_alg_hash: Bits256([0; 32]),
            r: Bits256([0; 32]),
            s: Bits256([0; 32]),
            v: 0,
        };

        let mut inputs = Vec::new();
        inputs.push(input);

        contract
            .methods()
            .update_temporal_numeric_values_v1(inputs)
            .call()
            .await
            .unwrap();
    }

    pub async fn get_temporal_value(
        contract: &StorkCore<WalletUnlocked>,
        feed_id: Bits256,
    ) -> TemporalNumericValue {
        contract
            .methods()
            .get_temporal_numeric_value_unchecked_v1(feed_id)
            .call()
            .await
            .unwrap()
            .value
    }
}
