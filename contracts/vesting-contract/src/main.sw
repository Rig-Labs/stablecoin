contract;

dep data_structures;
dep interface;
dep utils;

use data_structures::{Asset, VestingSchedule};
use interface::VestingContract;
use utils::{calculate_redeemable_amount};
use std::{
    address::Address,
    auth::msg_sender,
    block::timestamp,
    logging::log,
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    admin: Identity = Identity::Address(Address::from(ZERO_B256)),
    vesting_schedules: StorageMap<Identity, Option<VestingSchedule>> = StorageMap {},
    vesting_addresses: StorageVec<Identity> = StorageVec {},
    asset: Asset = Asset::new(ContractId::from(ZERO_B256), 0),
}

// pub fn get_msg_sender_address_or_panic() -> Address {
//     let sender: Result<Identity, AuthError> = msg_sender();
//     if let Identity::Address(address) = sender.unwrap() {
//         address
//     } else {
//         revert(0);
//     }
// }
// #[storage(read)]
// fn validate_admin() {
//     let sender = get_msg_sender_address_or_panic();
//     require(storage.admin == sender, "Access denied");
// }
impl VestingContract for Contract {
    #[storage(write, read)]
    fn constructor(
        admin: Identity,
        schedules: Vec<VestingSchedule>,
        asset: Asset,
    ) {
        storage.admin = admin;
        let mut i = 0;

        while i < schedules.len() {
            let schedule = schedules.get(i).unwrap();

            let existing_schedule = storage.vesting_schedules.get(schedule.recipient);
            require(existing_schedule.is_none(), "Schedule already exists");

            storage.vesting_schedules.insert(schedule.recipient, Option::Some(schedule));
            storage.vesting_addresses.push(schedule.recipient);
            i += 1;
        }
        storage.asset = asset;
    }

    #[storage(read, write)]
    fn claim_vested_tokens(address: Identity) {
        // TODO add re entry guard
        let mut schedule = storage.vesting_schedules.get(address).unwrap();
        let now = timestamp();

        let unclaimed = calculate_redeemable_amount(now, schedule);
        require(unclaimed > 0, "Nothing to redeem");

        transfer(unclaimed, storage.asset.id, address);
        schedule.claimed_amount += unclaimed;

        storage.vesting_schedules.insert(address, Option::Some(schedule));
    }

    #[storage(read)]
    fn get_vesting_schedule(address: Identity) -> Option<VestingSchedule> {
        return storage.vesting_schedules.get(address);
    }


    // #[storage(read)]
    // fn get_vesting_addresses() -> Vec<Identity> {
    //     let mut i = 0;
    //     let mut addresses: Vec<Identity> = Vec::new();
    //     while i < storage.vesting_addresses.len() {
    //         let address = storage.vesting_addresses.get(i).unwrap();
    //         addresses.push(address);
    //         i += 1;
    //     }
    //     return addresses;
    // }
    #[storage(read)]
    fn get_redeemable_amount(at_timestamp: u64, address: Identity) -> u64 {
        let schedule = storage.vesting_schedules.get(address).unwrap();

        return calculate_redeemable_amount(at_timestamp, schedule);
    }
}
