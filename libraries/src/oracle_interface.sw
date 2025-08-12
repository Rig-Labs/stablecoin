library;

use std::bytes::Bytes;

pub struct Price {
    pub value: u64,
    pub time: u64,
}

/// Oracle configuration for Stork, Pyth, and Redstone
/// They're separate as Stork/Pyth use b256 for feeds while Redstone uses u256.
pub struct StorkConfig {
    /// Contract address
    pub contract_id: ContractId,
    /// Price feed ID
    pub feed_id: b256,
    /// Precision
    pub precision: u32,
}

pub struct PythConfig {
    /// Contract address
    pub contract_id: ContractId,
    /// Price feed ID
    pub feed_id: b256,
    /// Precision
    pub precision: u32,
}

pub struct RedstoneConfig {
    /// Contract address
    pub contract_id: ContractId,
    /// Price feed ID
    pub feed_id: u256,
    /// Precision
    pub precision: u32,
}


// Oracle interface
abi Oracle {
    /// ------------------------ MAIN FUNCTIONS ------------------------ ///

    // Get the price from the configured oracles.
    #[storage(read, write)]
    fn get_price() -> u64;

    /// ------------------------ PUBLIC SETTERS ------------------------ ///

    // Testing workaround
    #[storage(write)]
    fn set_debug_timestamp(timestamp: u64);

    // Set the stork config.
    #[storage(read, write)]
    fn set_stork_config(config: StorkConfig);

    // Set the pyth config.
    #[storage(read, write)]
    fn set_pyth_config(config: PythConfig);

    // Set the redstone config.
    #[storage(read, write)]
    fn set_redstone_config(config: RedstoneConfig);

    /// ------------------------ PUBLIC GETTERS ------------------------ ///

    // Get the stork config.
    #[storage(read)]
    fn get_stork_config() -> StorkConfig;

    // Get the pyth config.
    #[storage(read)]
    fn get_pyth_config() -> PythConfig;

    // Get the redstone config.
    #[storage(read)]
    fn get_redstone_config() -> RedstoneConfig;
}

impl Price {
    pub fn new(price: u64, time: u64) -> Self {
        Self {
            value: price,
            time,
        }
    }
}

// Mocked Redstone structures
abi RedstoneCore {
    #[storage(read)]
    fn read_prices(feed_ids: Vec<u256>) -> Vec<u256>;

    // Testing only, not the actual function signature of redstone
    #[storage(write)]
    fn write_prices(feed: Vec<(u256, u256)>);

    #[storage(read)]
    fn read_timestamp() -> u64;

    // Purely for setting up testing conditions
    #[storage(write)]
    fn set_timestamp(time: u64);
}
