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
    call_frames::msg_asset_id,
    context::msg_amount,
    logging::log,
    storage::StorageMap,
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    admin: Identity = Identity::Address(Address::from(ZERO_B256)),
    vesting_schedules: StorageMap<Identity, Option<VestingSchedule>> = StorageMap {},
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
            i += 1;
        }
        storage.asset = asset;
    }

    #[storage(read)]
    fn get_vesting_schedule(address: Identity) -> Option<VestingSchedule> {
        return storage.vesting_schedules.get(address);
    }

    #[storage(read)]
    fn get_redeemable_amount(now: u64, address: Identity) -> u64 {
        let schedule = storage.vesting_schedules.get(address).unwrap();

        return calculate_redeemable_amount(now, schedule);
    }

    #[storage(read, write)]
    fn claim_vested_tokens(address: Identity) {
        // TODO add re entry guard
        let mut schedule = storage.vesting_schedules.get(address).unwrap();
        let now = timestamp();

        let redeemable_amount = calculate_redeemable_amount(now, schedule);
        let unclaimed = redeemable_amount - schedule.claimed_amount;
        require(unclaimed > 0, "Nothing to redeem");

        transfer(unclaimed, storage.asset.id, address);
        schedule.claimed_amount = redeemable_amount;

        storage.vesting_schedules.insert(address, Option::Some(schedule));
        // Update schedule
    }
}
