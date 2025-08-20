use fuels::prelude::{Address, Bech32Address};
use fuels::types::errors::Error;
use fuels::types::AssetId;
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

pub const TOTAL_AMOUNT_VESTED: u64 = 100_000_000 * 68 / 100 * PRECISION;

pub mod vesting_abi {
    use fuels::types::transaction_builders::VariableOutputPolicy;

    use crate::data_structures::ContractInstance;

    use super::*;
    pub async fn instantiate_vesting_contract<T: Account + Clone>(
        contract: &ContractInstance<VestingContract<T>>,
        asset_contract: &AssetId,
        schedules: Vec<VestingSchedule>,
        debug: bool,
    ) -> Result<CallResponse<()>, Error> {
        contract
            .contract
            .methods()
            .constructor(asset_contract.clone().into(), schedules, debug)
            .with_contract_ids(&[contract.implementation_id.into()])
            .call()
            .await
    }

    pub async fn set_timestamp<T: Account + Clone>(
        contract: &ContractInstance<VestingContract<T>>,
        timestamp: u64,
    ) -> Result<CallResponse<()>, Error> {
        contract
            .contract
            .methods()
            .set_current_time(timestamp)
            .with_contract_ids(&[contract.implementation_id.into()])
            .call()
            .await
    }
    pub async fn get_vesting_schedule_call<T: Account + Clone>(
        contract: &ContractInstance<VestingContract<T>>,
        recipient: Identity,
    ) -> Result<CallResponse<VestingSchedule>, Error> {
        contract
            .contract
            .methods()
            .get_vesting_schedule(recipient)
            .with_contract_ids(&[contract.implementation_id.into()])
            .call()
            .await
    }

    pub async fn get_redeemable_amount<T: Account + Clone>(
        contract: &ContractInstance<VestingContract<T>>,
        timestamp: u64,
        recipient: Identity,
    ) -> Result<CallResponse<u64>, Error> {
        contract
            .contract
            .methods()
            .get_redeemable_amount(timestamp, recipient)
            .with_contract_ids(&[contract.implementation_id.into()])
            .call()
            .await
    }

    pub async fn claim_vested_tokens<T: Account + Clone>(
        contract: &ContractInstance<VestingContract<T>>,
    ) -> Result<CallResponse<()>, Error> {
        contract
            .contract
            .methods()
            .claim_vested_tokens()
            .with_contract_ids(&[contract.implementation_id.into()])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }
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
