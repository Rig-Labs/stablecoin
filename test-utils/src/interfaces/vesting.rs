use std::{fs::File, io::BufReader, str::FromStr};

use fuels::prelude::{Address, Bech32Address};
use fuels::types::AssetId;
use fuels::{
    prelude::{abigen, Account},
    programs::responses::CallResponse,
    types::Identity,
};
use serde::Deserialize;

use crate::setup::common::get_absolute_path_from_relative;

abigen!(Contract(
    name = "VestingContract",
    abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
));

pub async fn instantiate_vesting_contract<T: Account>(
    contract: &VestingContract<T>,
    asset_contract: &AssetId,
    schedules: Vec<VestingSchedule>,
) -> CallResponse<()> {
    contract
        .methods()
        .constructor(asset_contract.clone().into(), schedules, true)
        .call()
        .await
        .unwrap()
}

pub async fn set_timestamp<T: Account>(
    contract: &VestingContract<T>,
    timestamp: u64,
) -> CallResponse<()> {
    contract
        .methods()
        .set_current_time(timestamp)
        .call()
        .await
        .unwrap()
}

pub fn load_vesting_schedules_from_json_file(path: &str) -> Vec<VestingSchedule> {
    let absolute_path = get_absolute_path_from_relative(path);

    let file = File::open(&absolute_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    // Parse the JSON into a Vec<IntermediateVestingSchedule>
    let schedules: Vec<IntermediateVestingSchedule> = serde_json::from_reader(reader).unwrap();

    // Convert Vec<IntermediateVestingSchedule> into Vec<VestingSchedule>
    let vesting_schedules: Vec<VestingSchedule> = schedules
        .into_iter()
        .map(|schedule| {
            get_vesting_schedule(
                schedule.cliff_amount,
                schedule.cliff_timestamp,
                schedule.end_timestamp,
                schedule.claimed_amount,
                schedule.total_amount,
                Identity::Address(Address::from(
                    Bech32Address::from_str(&schedule.recipient).unwrap(),
                )),
            )
        })
        .collect();

    vesting_schedules
}

pub fn get_vesting_schedule(
    cliff_amount: u64,
    cliff_timestamp: u64,
    end_timestamp: u64,
    claimed_amount: u64,
    total_amount: u64,
    recipient: Identity,
) -> VestingSchedule {
    VestingSchedule {
        cliff_amount,
        cliff_timestamp,
        end_timestamp,
        claimed_amount,
        total_amount,
        recipient,
    }
}

// Intermediary struct matching the JSON structure
#[derive(Debug, Deserialize)]
struct IntermediateVestingSchedule {
    cliff_amount: u64,
    cliff_timestamp: u64,
    end_timestamp: u64,
    claimed_amount: u64,
    total_amount: u64,
    recipient: String, // Assume the JSON contains a string for the recipient.
}
