contract;
dep utils;
use libraries::community_issuance_interface::{CommunityIssuance};
use libraries::fluid_math::{null_contract, null_identity_address,  DECIMAL_PRECISION, dec_pow,};
use utils::*;


use std::{
    auth::{
        AuthError,
        msg_sender,
    },
    block::{
        height,
        timestamp,
    },
    identity::{Identity},
    call_frames::{contract_id, msg_asset_id},
    context::{balance_of, msg_amount},
    contract_id::ContractId,
    revert::require,
    token::transfer,
    u256::U256,
    u128::U128,
    logging::log,
};

storage {
    stability_pool_contract: ContractId = null_contract(),
    fpt_token_contract: ContractId = null_contract(),
    is_initialized: bool = false,
    total_fpt_issued: u64 = 0,
    deployment_time: u64 = 0,
    admin: Identity = null_identity_address(),
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
        fpt_token_contract: ContractId,
        admin: Identity,
        debugging: bool,
    ) {
        require(!storage.is_initialized, "Contract is already initialized");
        storage.stability_pool_contract = stability_pool_contract;
        storage.fpt_token_contract = fpt_token_contract;
        storage.is_initialized = true;
        storage.admin = admin;
        storage.debug = debugging;
        storage.deployment_time = internal_get_current_time();
    }

    #[storage(read, write)]
    fn start_rewards_increase_transition(total_transition_time_seconds: u64) {
        internal_require_caller_is_admin();
        require(!storage.has_transitioned_rewards, "Rewards have already transitioned");
        require(total_transition_time_seconds > 60*60*24*7, "Total transition time must be greater than 1 week");
        storage.has_transitioned_rewards = true;
        storage.time_transition_started = internal_get_current_time();
        storage.total_transition_time_seconds = total_transition_time_seconds;
    }

    #[storage(write, read)]
    fn public_start_rewards_increase_transition_after_deadline(){
        require(!storage.has_transitioned_rewards, "Rewards have already transitioned");
        let time_since_started_rewards = internal_get_current_time() - storage.deployment_time;
        require(time_since_started_rewards > 60*60*24*30*12, "Rewards can only be publicly increased after 1 year of inactivity"); // 1 year
        let total_transition_time_seconds = 60*60*24*30*6; // 6 months
        storage.has_transitioned_rewards = true;
        storage.time_transition_started = internal_get_current_time();
        storage.total_transition_time_seconds = total_transition_time_seconds;
    }

    #[storage(read, write)]
    fn issue_fpt() -> u64 {
        internal_require_caller_is_stability_pool();
        let latest_total_fpt_issued = ((U128::from_u64(internal_get_fpt_supply_cap(storage.time_transition_started, storage.total_transition_time_seconds, internal_get_current_time(), storage.has_transitioned_rewards)) * U128::from_u64(internal_get_cumulative_issuance_fraction(internal_get_current_time(), storage.deployment_time))) / U128::from_u64(DECIMAL_PRECISION)).as_u64().unwrap();
        let issuance = latest_total_fpt_issued - storage.total_fpt_issued;
        storage.total_fpt_issued = latest_total_fpt_issued;
        issuance
    }

    #[storage(read)]
    fn send_fpt(account: Identity, amount: u64) {
        internal_require_caller_is_stability_pool();
        transfer(amount, storage.fpt_token_contract, account);
    }

    #[storage(read)]
    fn get_current_time() -> u64 {
        return internal_get_current_time();
    }

    #[storage(write, read)]
    fn set_current_time(time: u64) {
        require(storage.debug, "Debugging must be enabled to set current time");
        storage.debug_timestamp = time;
    }
}

#[storage(read)]
fn internal_require_caller_is_stability_pool() {
    require(msg_sender().unwrap() == Identity::ContractId(storage.stability_pool_contract), "Caller must be stability pool");
}

#[storage(read)]
fn internal_require_caller_is_admin() {
    require(msg_sender().unwrap() == storage.admin, "Caller must be admin");
}

#[storage(read)]
fn internal_get_current_time() -> u64 {
    if storage.debug {
        return storage.debug_timestamp;
    } else {
        return timestamp();
    }
}