contract;

use libraries::comunity_issuance_interface::{CommunityIssuance};
use libraries::fluid_math::{null_contract, null_identity_address, dec_pow, fm_min};

use std::{
    address::*,
    auth::{
        AuthError,
        msg_sender,
    },
    identity::{Identity},
    call_frames::{contract_id, msg_asset_id},
    context::{balance_of, msg_amount},
    contract_id::ContractId,
    revert::require,
    storage::*,
    token::*,
};

// todo: if sway supports constant multiplications inherit decimal precision from fluid math
const FPT_SUPPLY_CAP = 32_000_000_000_000_000;
const SECONDS_IN_ONE_MINUTE = 60;
const ISSUANCE_FACTOR = 999998681227695000;

storage {
    stability_pool_contract: ContractId,
    fpt_token_contract: ContractId,
    is_initialized: bool = false,
    total_fpt_issued: u64 = 0,
    deployment_time: u64 = 0,
    admin: Identity = Identity::Address(Address::from(ZERO_B256)),
    debug: bool = false,
    debug_timestamp: u64 = 0,
    has_transitioned_rewards bool = false,
    time_transition_started: u64 = 0,
    total_transition_time_seconds u64 = 0,
}

impl CommunityIssuance for Contract {
    #[storage(read, write)]
    fn initialize(
        stability_pool_contract: ContractId,
        fpt_token_contract: ContractId,
        admin: Identity,
        debugging: bool,
        time: u64,
    ) {
        require(!storage.is_initialized, "Contract is already initialized");
        storage.stability_pool_contract = stability_pool_contract;
        storage.fpt_token_contract = fpt_token_contract;
        storage.is_initialized = true;
        storage.deployment_time = now();
        storage.admin = admin;
        storage.debug = debugging;
        if (storage.debugging){
            storage.debug_timestamp = time;
        }
        storage.deployment_time = internal_get_current_time();

    }

    #[storage(read, write)]
    fn start_rewards_increase_transition(total_transition_time_seconds: u64) {
        internal_require_caller_is_admin();
        require(!storage.has_transitioned_rewards, "Rewards have already transitioned");
        storage.has_transitioned_rewards = true;
        storage.time_transition_started = internal_get_current_time();
        storage.total_transition_time_seconds = total_transition_time_seconds;
    }

    #[storage(read, write)]
    fn issue_fpt() -> u64 {
        internal_require_caller_is_stability_pool();
        let latest_total_fpt_issued = (internal_get_fpt_supply_cap() * internal_get_cumulative_issuance_fraction()) / DECIMAL_PRECISION;
        let issuance = latest_total_fpt_issued - storage.total_fpt_issued;
        storage.total_fpt_issued = latest_total_fpt_issued;
        issuance
    }

    #[storage(read, write)]
    fn send_fpt(account: Address, amount: u64) {
        internal_require_caller_is_stability_pool();
        transfer(amount, storage.fpt_token_contract, account);
    }

    #[storage(read)]
    fn get_current_time() -> u64 {
        return internal_get_current_time();
    }

    #[storage(write, read)]
    fn set_current_time(time: u64) {
        internal_require_caller_is_admin();
        require(storage.debug, "Debugging must be enabled to set current time");
        storage.debug_timestamp = time;
    }
}

// todo: do we need to use u128?
#[storage(read)]
fn internal_get_fpt_supply_cap() -> u64 {
    if (!has_transitioned_rewards){
        FPT_SUPPLY_CAP / 2;
    } else {
        let time_since_transition_started_seconds = internal_get_current_time() - storage.time_transition_started;
        // if we just input time in seconds is easier than putting in an end timestamp and subtracting it here by the start timestamp?
        let change_in_fpt_supply_cap = time_since_transition_started_seconds / storage.total_transition_time_seconds;
        (FPT_SUPPLY_CAP / 2) + (fm_min(1, change_in_fpt_supply_cap) * (FPT_SUPPLY_CAP / 2));
    }
}

#[storage(read)]
fn internal_get_cumulative_issuance_fraction() -> u64 {
    let time_passed_in_minutes = internal_get_current_time() - storage.deployment_time / SECONDS_IN_ONE_MINUTE;

    let power = dec_pow(ISSUANCE_FACTOR, time_passed_in_minutes);

    let cumulative_issuance_fraction = DECIMAL_PRECISION - power;

    require(cumulativeIssuanceFraction <= DECIMAL_PRECISION, "Cumulative issuance fraction must be in range [0,1]");

    cumulative_issuance_fraction
}

#[storage(read)]
fn internal_require_caller_is_stability_pool() -> {
    require(msg_sender().unwrap() == storage.stability_pool_contract, "Caller must be stability pool");
}

#[storage(read)]
fn internal_require_caller_is_admin() -> {
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