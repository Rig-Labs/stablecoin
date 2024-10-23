use csv::ReaderBuilder;
use fuels::prelude::{Address, Bech32Address};
use fuels::types::errors::Error;
use fuels::types::{AssetId, ContractId};
use fuels::{
    prelude::{abigen, Account},
    programs::responses::CallResponse,
    types::Identity,
};
use serde::Deserialize;
use std::{fs::File, io::BufReader, str::FromStr};

use crate::data_structures::PRECISION;
use crate::setup::common::get_absolute_path_from_relative;

abigen!(Contract(
    name = "VestingContract",
    abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
));

const TOTAL_AMOUNT_VESTED: u64 = 100_000_000 * 68 / 100 * PRECISION;

pub async fn instantiate_vesting_contract<T: Account>(
    contract: &VestingContract<T>,
    asset_contract: &AssetId,
    schedules: Vec<VestingSchedule>,
    debug: bool,
) -> Result<CallResponse<()>, Error> {
    contract
        .methods()
        .constructor(asset_contract.clone().into(), schedules, debug)
        .call()
        .await
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

pub fn load_vesting_schedules_from_csv(
    path: &str,
    cliff_percentage: f64,
    seconds_to_cliff: u64,
    seconds_vesting_duration: u64,
) -> Vec<VestingSchedule> {
    let absolute_path = get_absolute_path_from_relative(path);
    let file = File::open(&absolute_path).expect("Failed to open file");
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let cliff_timestamp = now_unix + seconds_to_cliff;
    let end_timestamp = cliff_timestamp + seconds_vesting_duration;

    let cliff_timestamp = tai64::Tai64::from_unix(cliff_timestamp.try_into().unwrap());
    let end_timestamp = tai64::Tai64::from_unix(end_timestamp.try_into().unwrap());

    let mut schedules = Vec::new();

    for result in reader.records() {
        let record = result.expect("Failed to read CSV record");
        if record.len() < 5 || record[1].is_empty() {
            continue; // Skip empty or invalid rows
        }

        println!("record: {:?}", record);

        let total_amount =
            (record[1].replace([',', '"'], "").parse::<f64>().unwrap() * PRECISION as f64) as u64;
        let recipient = if !record[2].is_empty() {
            Identity::Address(Address::from_str(&record[2]).unwrap())
        } else if !record[3].is_empty() {
            continue;
            // ignore the recipient for now since ETH addresses are not supported yet
            // Identity::Address(Address::from_str(&record[3]).unwrap())
        } else {
            continue; // Skip if both wallet addresses are empty
        };

        let schedule = VestingSchedule {
            cliff_amount: (total_amount as f64 * cliff_percentage) as u64,
            cliff_timestamp: cliff_timestamp.0,
            end_timestamp: end_timestamp.0,
            claimed_amount: 0,
            total_amount,
            recipient,
        };

        println!("schedule: {:?}", schedule);

        schedules.push(schedule);
    }
    // take the sum of all total_amounts
    let total_sum: u64 = schedules.iter().map(|s| s.total_amount).sum();
    println!("Total sum of all vesting amounts: {}", total_sum);
    // add one more schedule with the remaining amount
    // TODO: find the total amount vested
    schedules.push(VestingSchedule {
        cliff_amount: total_sum,
        cliff_timestamp: cliff_timestamp.0,
        end_timestamp: end_timestamp.0,
        claimed_amount: 0,
        total_amount: TOTAL_AMOUNT_VESTED - total_sum,
        recipient: Identity::ContractId(
            ContractId::from_str(
                "0x4761863a5b9a7ec3263964f694f453a5a67cf0d458ebc3e36eb618d43809c785",
            )
            .unwrap(),
        ),
    });
    schedules
}
