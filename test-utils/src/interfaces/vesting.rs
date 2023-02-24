use fuels::{
    prelude::{abigen, Address, ContractId},
    programs::call_response::FuelCallResponse,
    types::Identity,
};

abigen!(Contract(
    name = "VestingContract",
    abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
));

pub async fn instantiate_vesting_contract(
    contract: &VestingContract,
    admin: &Address,
    vesting_schedule: &Vec<VestingSchedule>,
    asset_contract: &ContractId,
    amount: u64,
) -> FuelCallResponse<()> {
    let asset: Asset = Asset {
        id: asset_contract.clone(),
        amount,
    };

    contract
        .methods()
        .constructor(
            Identity::Address(admin.clone()),
            vesting_schedule.clone(),
            asset.clone(),
        )
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
    revocable: bool,
) -> VestingSchedule {
    VestingSchedule {
        cliff_amount,
        cliff_timestamp,
        end_timestamp,
        claimed_amount,
        total_amount,
        recipient,
        revocable,
    }
}
