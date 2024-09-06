contract;
// This contract, CommunityIssuance, manages the issuance of FPT (Fluid Protocol Token) to the Stability Pool.
// It controls the distribution of FPT over time, implementing a decay function for token release.
//
// Key functionalities include:
// - Initializing the contract with necessary parameters
// - Calculating and issuing FPT based on time elapsed since deployment
// - Managing the transition of rewards distribution from an initial rapid issuance to a more gradual long-term rate
// - Interfacing with the Stability Pool to send FPT
// - Providing admin functions for contract management
//
// The contract uses a mathematical model to determine token issuance, ensuring a controlled
// and predictable distribution of FPT to incentivize participation in the Stability Pool.
// It also handles the transition period between different issuance rates, allowing for
// a smooth change in the token distribution strategy over time.


mod utils;

use libraries::community_issuance_interface::CommunityIssuance;
use libraries::fluid_math::{dec_pow, DECIMAL_PRECISION, fm_multiply_ratio};
use ::utils::*;
use std::{
    asset::transfer,
    auth::{
        AuthError,
    },
    block::{
        height,
        timestamp,
    },
    call_frames::{
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    u128::U128,
};

const ONE_WEEK_IN_SECONDS: u64 = 604_800;
const SIX_MONTHS_IN_SECONDS: u64 = 15_780_000;
const ONE_YEAR_IN_SECONDS: u64 = 31_104_000;

storage {
    stability_pool_contract: ContractId = ContractId::zero(),
    fpt_token_contract: AssetId = AssetId::zero(),
    is_initialized: bool = false,
    total_fpt_issued: u64 = 0,
    deployment_time: u64 = 0,
    admin: Identity = Identity::Address(Address::zero()),
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
        require(
            !storage
                .is_initialized
                .read(),
            "CommunityIssuance: Contract is already initialized",
        );
        storage
            .stability_pool_contract
            .write(stability_pool_contract);
        storage.fpt_token_contract.write(fpt_token_contract);
        storage.is_initialized.write(true);
        storage.admin.write(admin);
        storage.debug.write(debugging);
        storage.deployment_time.write(internal_get_current_time());
    }

    #[storage(read, write)]
    fn start_rewards_increase_transition(total_transition_time_seconds: u64) {
        internal_require_caller_is_admin();
        require(
            !storage
                .has_transitioned_rewards
                .read(),
            "CommunityIssuance: Rewards have already transitioned",
        );
        require(
            total_transition_time_seconds > ONE_WEEK_IN_SECONDS,
            "CommunityIssuance: Total transition time must be greater than 1 week",
        );
        storage.has_transitioned_rewards.write(true);
        storage
            .time_transition_started
            .write(internal_get_current_time());
        storage
            .total_transition_time_seconds
            .write(total_transition_time_seconds);
    }

    #[storage(write, read)]
    fn public_start_rewards_increase_transition_after_deadline() {
        require(
            !storage
                .has_transitioned_rewards
                .read(),
            "CommunityIssuance: Rewards have already transitioned",
        );
        let time_since_started_rewards = internal_get_current_time() - storage.deployment_time.read();
        require(
            time_since_started_rewards > ONE_YEAR_IN_SECONDS,
            "CommunityIssuance: Rewards can only be publicly increased after 1 year of inactivity",
        ); // 1 year
        let total_transition_time_seconds = SIX_MONTHS_IN_SECONDS; // 6 months
        storage.has_transitioned_rewards.write(true);
        storage
            .time_transition_started
            .write(internal_get_current_time());
        storage
            .total_transition_time_seconds
            .write(total_transition_time_seconds);
    }

    #[storage(read, write)]
    fn issue_fpt() -> u64 {
        internal_require_caller_is_stability_pool();

        let latest_total_fpt_issued = fm_multiply_ratio(
            internal_get_fpt_supply_cap(
                storage
                    .time_transition_started
                    .read(),
                storage
                    .total_transition_time_seconds
                    .read(),
                internal_get_current_time(),
                storage
                    .has_transitioned_rewards
                    .read(),
            ),
            internal_get_cumulative_issuance_fraction(internal_get_current_time(), storage.deployment_time.read()),
            DECIMAL_PRECISION,
        );
        let issuance = latest_total_fpt_issued - storage.total_fpt_issued.read();
        storage.total_fpt_issued.write(latest_total_fpt_issued);
        return issuance
    }

    #[storage(read)]
    fn send_fpt(account: Identity, amount: u64) {
        internal_require_caller_is_stability_pool();
        if amount > 0 {
            transfer(account, storage.fpt_token_contract.read(), amount);
        }
    }

    #[storage(read)]
    fn get_current_time() -> u64 {
        return internal_get_current_time();
    }

    #[storage(write, read)]
    fn set_current_time(time: u64) {
        require(
            storage
                .debug
                .read(),
            "CommunityIssuance: Debugging must be enabled to set current time",
        );
        storage.debug_timestamp.write(time);
    }
}
#[storage(read)]
fn internal_require_caller_is_stability_pool() {
    require(
        msg_sender()
            .unwrap() == Identity::ContractId(storage.stability_pool_contract.read()),
        "CommunityIssuance: Caller must be stability pool",
    );
}

#[storage(read)]
fn internal_require_caller_is_admin() {
    require(
        msg_sender()
            .unwrap() == storage
            .admin
            .read(),
        "CommunityIssuance: Caller must be admin",
    );
}

#[storage(read)]
fn internal_get_current_time() -> u64 {
    if storage.debug.read() {
        return storage.debug_timestamp.read();
    } else {
        return timestamp();
    }
}
