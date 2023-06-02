use fuels::{
    prelude::{abigen, Account, Address, ContractId},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "VestingContract",
    abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
));

pub async fn instantiate_vesting_contract<T: Account>(
    contract: &VestingContract<T>,
    admin: &Address,
    asset_contract: &ContractId,
) -> FuelCallResponse<()> {
    contract
        .methods()
        .constructor(
            Identity::Address(admin.clone()),
            asset_contract.clone(),
            true,
        )
        .call()
        .await
        .unwrap()
}

pub async fn initiate_vesting_schedules<T: Account>(
    contract: &VestingContract<T>,
    schedules: Vec<VestingSchedule>,
) -> FuelCallResponse<()> {
    contract
        .methods()
        .initiate_vesting_schedules(schedules)
        .call()
        .await
        .unwrap()
}

pub async fn set_timestamp<T: Account>(
    contract: &VestingContract<T>,
    timestamp: u64,
) -> FuelCallResponse<()> {
    contract
        .methods()
        .set_current_time(timestamp)
        .call()
        .await
        .unwrap()
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
