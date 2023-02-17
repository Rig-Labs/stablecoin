contract;

dep data_structures;
dep interface;

use data_structures::{VestingSchedule};
use interface::VestingContract;
use std::{storage::StorageVec};

storage {
    price: u64 = 0,
    precision: u64 = 6,
    vesting_schedules: StorageVec<VestingSchedule> = StorageVec {},
}

impl VestingContract for Contract {
    #[storage(read)]
    fn get_price() -> u64 {
        storage.price
    }

    #[storage(read)]
    fn get_precision() -> u64 {
        storage.precision
    }

    #[storage(write)]
    fn set_price(_price: u64) {
        storage.price = _price
    }

    #[storage(read)]
    fn get_vesting_schedules() -> Vec<VestingSchedule> {
        let mut vec: Vec<VestingSchedule> = Vec::new();
        let mut i = 0;
        while i < storage.vesting_schedules.len() {
            vec.push(storage.vesting_schedules.get(i).unwrap());
            i += 1;
        }
        return vec;
    }
}
