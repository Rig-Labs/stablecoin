contract;
// This contract, MockStork, is a mock implementation of the Stork oracle interface.
// It is used for testing and simulation purposes within the Fluid Protocol.
//
// Key functionalities include:
// - Simulating price feeds for testing purposes
// - Providing a mock interface for interacting with the Stork oracle
// - Ensuring compatibility with the Stork oracle interface for testing
//
// To the auditor: This contract is not used in the system. It is only used for testing.

use std::{block::timestamp, hash::Hash, string::String, vm::evm::evm_address::EvmAddress};
use sway_libs::signed_integers::i128::I128;
use libraries::{
    stork_interface::TemporalNumericValueInput,
    temporal_numeric_value::TemporalNumericValue
};

storage {
    latest_temporal_values: StorageMap<b256, TemporalNumericValue> = StorageMap {},
}

abi Stork {
    // Only implementing the required functions for testing
    #[storage(read, write), payable]
    fn update_temporal_numeric_values_v1(update_data: Vec<TemporalNumericValueInput>);
    
    #[storage(read)]
    fn get_temporal_numeric_value_unchecked_v1(id: b256) -> TemporalNumericValue;
}

impl Stork for Contract {
    #[storage(read, write), payable]
    fn update_temporal_numeric_values_v1(update_data: Vec<TemporalNumericValueInput>) {
        // Simplified mock implementation - just stores the values without signature verification
        let mut index = 0;
        while index < update_data.len() {
            let data = update_data.get(index).unwrap();
            storage
                .latest_temporal_values
                .insert(
                    data.id,
                    data.temporal_numeric_value,
                );
            index += 1;
        }
    }
    
    #[storage(read)]
    fn get_temporal_numeric_value_unchecked_v1(id: b256) -> TemporalNumericValue {
        let value = storage.latest_temporal_values.get(id).try_read();
        require(value.is_some(), "Temporal numeric value not found");
        
        return value.unwrap();
    }
}