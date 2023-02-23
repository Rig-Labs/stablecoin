contract;

dep data_structures;

use data_structures::{LocalVariables_OpenTrove};

use libraries::data_structures::{Status};
use libraries::trove_manager_interface::{TroveManager};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::fluid_math::*;

use std::{
    address::Address,
    auth::msg_sender,
    block::{
        height,
        timestamp,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    logging::log,
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    trove_manager_contract: ContractId = ContractId::from(ZERO_B256),
    sorted_troves_contract: ContractId = ContractId::from(ZERO_B256),
    oracle_contract: ContractId = ContractId::from(ZERO_B256),
    usdf: ContractId = ContractId::from(ZERO_B256),
    fpt_staking_contract: ContractId = ContractId::from(ZERO_B256),
}

impl BorrowOperations for Contract {
    #[storage(read, write)]
    fn open_trove(
        _max_fee_percentage: u64,
        _usdf_amount: u64,
        _upper_hint: Identity,
        _lower_hint: Identity,
    ) {
        require_valid_max_fee_percentage(_max_fee_percentage);
        // TODO Rqure Trove is not active / exists
        let mut vars = LocalVariables_OpenTrove::new();
        vars.net_debt = _usdf_amount;

        vars.usdf_fee = internal_trigger_borrowing_fee();
        vars.net_debt = vars.net_debt + vars.usdf_fee;

        require_at_least_min_net_debt(vars.net_debt);

        // ICR is based on the composite debt, i.e. the requested LUSD amount + LUSD borrowing fee + LUSD gas comp.
        vars.composite_debt = get_composite_debt(vars.net_debt);
        require(vars.composite_debt > 0, "BorrowOperations: composite debt must be greater than 0");
        let trove_manager = abi(TroveManager, storage.trove_manager_contract.value);

        let sender = msg_sender().unwrap();

        trove_manager.set_trove_status(sender, Status::Active);
        trove_manager.increase_trove_coll(sender, msg_amount());
        trove_manager.increase_trove_debt(sender, vars.composite_debt);
    }

    #[storage(read, write)]
    fn add_coll(upper_hint: Identity, lower_hint: Identity) {}

    #[storage(read, write)]
    fn move_asset_gain_to_trove(id: Identity, upper_hint: Identity, lower_hint: Identity) {}

    #[storage(read, write)]
    fn withdraw_coll(amount: u64, upper_hint: Identity, lower_hint: Identity) {}

    #[storage(read, write)]
    fn withdraw_usdf(
        max_fee: u64,
        amount: u64,
        upper_hint: Identity,
        lower_hint: Identity,
    ) {}

    #[storage(read, write)]
    fn repay_usdf(amount: u64, upper_hint: Identity, lower_hint: Identity) {}

    #[storage(read, write)]
    fn close_trove() {}

    #[storage(read, write)]
    fn adjust_trove(
        max_fee: u64,
        coll_withdrawl: u64,
        debt_change: u64,
        is_debt_increase: bool,
        upper_hint: Identity,
        lower_hint: Identity,
    ) {}

    #[storage(read, write)]
    fn claim_collateral() {}

    #[storage(read, write)]
    fn get_composite_debt(id: Identity) -> u64 {
        return 0
    }
}

fn internal_trigger_borrowing_fee() -> u64 {
    // TODO
    return 0
}

fn get_composite_debt(_net_debt: u64) -> u64 {
    return fm_get_net_debt(_net_debt);
}

fn require_at_least_min_net_debt(_net_debt: u64) {
    require(_net_debt > MCR, "BorrowOperations: net debt must be greater than 0");
}

fn require_valid_max_fee_percentage(_max_fee_percentage: u64) {
    require(_max_fee_percentage < DECIMAL_PRECISION, "BorrowOperations: max fee percentage must be less than 100");
}
