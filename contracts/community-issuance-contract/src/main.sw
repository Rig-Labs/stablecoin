contract;

mod utils;

use libraries::community_issuance_interface::{CommunityIssuance};
use libraries::fluid_math::{
    dec_pow,
    DECIMAL_PRECISION,
    fm_multiply_ratio,
    null_contract,
    null_identity_address,
    ZERO_B256,
};
use ::utils::*;

use std::{
    auth::{
        AuthError,
        msg_sender,
    },
    block::{
        height,
        timestamp,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    contract_id::ContractId,
    identity::{
        Identity,
    },
    logging::log,
    revert::require,
    token::transfer,
    u128::U128,
    u256::U256,

};

const ONE_WEEK_IN_SECONDS: u64 = 604800;
const SIX_MONTHS_IN_SECONDS: u64 = 15780000;
const ONE_YEAR_IN_SECONDS: u64 = 31104000;

storage {
    stability_pool_contract: ContractId = ContractId::from(ZERO_B256),
    fpt_token_contract: AssetId = ZERO_B256,
    is_initialized: bool = false,
    total_fpt_issued: u64 = 0,
    deployment_time: u64 = 0,
    admin: Identity = Identity::Address(Address::from(ZERO_B256)),
    debug: bool = false,
    debug_timestamp: u64 = 0,
    has_transitioned_rewards: bool = false,
    time_transition_started: u64 = 0,
    total_transition_time_seconds: u64 = 0,
}

impl CommunityIssuance for Contract {
    #[storage(read, write)]
    fn initialize(
        stability_pool_contract: ContractId,
        fpt_token_contract: AssetId,
        admin: Identity,
        debugging: bool,
    ) {
        require(!storage.is_initialized.read(), "Contract is already initialized");
        storage.stability_pool_contract.write(stability_pool_contract);
        storage.fpt_token_contract.write(fpt_token_contract);
        storage.is_initialized.write(true);
        storage.admin.write(admin);
        storage.debug.write(debugging);
        storage.deployment_time.write(internal_get_current_time());
    }

    #[storage(read, write)]
    fn start_rewards_increase_transition(total_transition_time_seconds: u64) {
        internal_require_caller_is_admin();
        require(!storage.has_transitioned_rewards.read(), "Rewards have already transitioned");
        require(total_transition_time_seconds > ONE_WEEK_IN_SECONDS, "Total transition time must be greater than 1 week");
        storage.has_transitioned_rewards.write(true);
        storage.time_transition_started.write(internal_get_current_time());
        storage.total_transition_time_seconds.write(total_transition_time_seconds);
    }

    #[storage(write, read)]
    fn public_start_rewards_increase_transition_after_deadline() {
        require(!storage.has_transitioned_rewards.read(), "Rewards have already transitioned");
        let time_since_started_rewards = internal_get_current_time() - storage.deployment_time.read();
        require(time_since_started_rewards > ONE_YEAR_IN_SECONDS, "Rewards can only be publicly increased after 1 year of inactivity"); // 1 year
        let total_transition_time_seconds = SIX_MONTHS_IN_SECONDS; // 6 months
        storage.has_transitioned_rewards.write(true);
        storage.time_transition_started.write(internal_get_current_time());
        storage.total_transition_time_seconds.write(total_transition_time_seconds);
    }

    #[storage(read, write)]
    fn issue_fpt() -> u64 {
        internal_require_caller_is_stability_pool();
        let latest_total_fpt_issued = fm_multiply_ratio(internal_get_fpt_supply_cap(storage.time_transition_started.read(), storage.total_transition_time_seconds.read(), internal_get_current_time(), storage.has_transitioned_rewards.read()), internal_get_cumulative_issuance_fraction(internal_get_current_time(), storage.deployment_time.read()), DECIMAL_PRECISION);
        let issuance = latest_total_fpt_issued - storage.total_fpt_issued.read();
        storage.total_fpt_issued.write(latest_total_fpt_issued);

        return issuance
    }

    #[storage(read)]
    fn send_fpt(account: Identity, amount: u64) {
        internal_require_caller_is_stability_pool();
        if amount > 0 {
            transfer(account , storage.fpt_token_contract.read(), amount);
        }
    }

    #[storage(read)]
    fn get_current_time() -> u64 {
        return internal_get_current_time();
    }

    #[storage(write, read)]
    fn set_current_time(time: u64) {
        require(storage.debug.read(), "Debugging must be enabled to set current time");
        storage.debug_timestamp.write(time);
    }
}

#[storage(read)]
fn internal_require_caller_is_stability_pool() {
    require(msg_sender().unwrap() == Identity::ContractId(storage.stability_pool_contract.read()), "Caller must be stability pool");
}

#[storage(read)]
fn internal_require_caller_is_admin() {
    require(msg_sender().unwrap() == storage.admin.read(), "Caller must be admin");
}

#[storage(read)]
fn internal_get_current_time() -> u64 {
    if storage.debug.read() {
        return storage.debug_timestamp.read();
    } else {
        return timestamp();
    }
}
