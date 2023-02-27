contract;

dep data_structures;

use data_structures::{LocalVariables_AdjustTrove, LocalVariables_OpenTrove};

use libraries::data_structures::{Status};
use libraries::token_interface::{Token};
use libraries::trove_manager_interface::{TroveManager};
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::{MockOracle};
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
    asset_contract: ContractId = ContractId::from(ZERO_B256),
    usdf_contract: ContractId = ContractId::from(ZERO_B256),
    fpt_staking_contract: ContractId = ContractId::from(ZERO_B256),
}

impl BorrowOperations for Contract {
    #[storage(read, write)]
    fn initialize(
        trove_manager_contract: ContractId,
        sorted_troves_contract: ContractId,
        oracle_contract: ContractId,
        asset_contract: ContractId,
        usdf_contract: ContractId,
        fpt_staking_contract: ContractId,
    ) {
        require(storage.trove_manager_contract.value == ZERO_B256, "BorrowOperations: contract is already initialized");

        storage.trove_manager_contract = trove_manager_contract;
        storage.sorted_troves_contract = sorted_troves_contract;
        storage.oracle_contract = oracle_contract;
        storage.asset_contract = asset_contract;
        storage.usdf_contract = usdf_contract;
        storage.fpt_staking_contract = fpt_staking_contract;
    }

    #[storage(read, write)]
    fn open_trove(
        _max_fee_percentage: u64,
        _usdf_amount: u64,
        _upper_hint: Identity,
        _lower_hint: Identity,
    ) {
        require_valid_asset_id();
        require_valid_max_fee_percentage(_max_fee_percentage);
        let oracle = abi(MockOracle, storage.oracle_contract.value);
        let trove_manager = abi(TroveManager, storage.trove_manager_contract.value);
        let sorted_troves = abi(SortedTroves, storage.sorted_troves_contract.value);
        let usdf = abi(Token, storage.usdf_contract.value);

        let mut vars = LocalVariables_OpenTrove::new();

        vars.net_debt = _usdf_amount;
        vars.price = oracle.get_price();

        // TODO Rqure Trove is not active / exists
        vars.usdf_fee = internal_trigger_borrowing_fee();
        vars.net_debt = vars.net_debt + vars.usdf_fee;

        require_at_least_min_net_debt(vars.net_debt);

        // ICR is based on the composite debt, i.e. the requested LUSD amount + LUSD borrowing fee + LUSD gas comp.
        require(vars.net_debt > 0, "BorrowOperations: composite debt must be greater than 0");

        let sender = msg_sender().unwrap();

        vars.icr = fm_compute_cr(msg_amount(), vars.net_debt, vars.price);
        vars.nicr = fm_compute_nominal_cr(msg_amount(), vars.net_debt);

        require_at_least_mcr(vars.icr);

        trove_manager.set_trove_status(sender, Status::Active);
        trove_manager.increase_trove_coll(sender, msg_amount());
        trove_manager.increase_trove_debt(sender, vars.net_debt);

        sorted_troves.insert(sender, vars.nicr, _upper_hint, _lower_hint);
        vars.array_index = trove_manager.add_trove_owner_to_array(sender);

        withdraw_usdf(sender, _usdf_amount, _usdf_amount);
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

// function _adjustTrove(address _borrower, uint _collWithdrawal, uint _LUSDChange, bool _isDebtIncrease, address _upperHint, address _lowerHint, uint _maxFeePercentage) internal {
#[storage(read)]
fn internal_adjust_trove(
    _borrower: Identity,
    _coll_withdrawal: u64,
    _lusd_change: u64,
    _is_debt_increase: bool,
    _upper_hint: Identity,
    _lower_hint: Identity,
    _max_fee_percentage: u64,
) {
    let oracle = abi(MockOracle, storage.oracle_contract.value);
    let trove_manager = abi(TroveManager, storage.trove_manager_contract.value);
    let price = oracle.get_price();

    let mut vars = LocalVariables_AdjustTrove::new();
    if _is_debt_increase {
        require_valid_max_fee_percentage(_max_fee_percentage);
        require_non_zero_debt_change(_lusd_change);
    }

    require_singular_coll_change(_coll_withdrawal);
    require_non_zero_adjustment(_coll_withdrawal, _lusd_change);

    let coll_res = internal_get_coll_change(msg_amount(), _coll_withdrawal);
    vars.coll_change = coll_res.0;
    vars.is_coll_increase = coll_res.1;

    vars.debt = trove_manager.get_trove_debt(_borrower);
    vars.coll = trove_manager.get_trove_coll(_borrower);
    // Vars stake trove manager update stake and total stakes
}

#[storage(read)]
fn require_non_zero_adjustment(_coll_withdrawl: u64, _lusd_change: u64) {
    let amount = msg_amount();
    if amount > 0 {
        require_valid_asset_id();
    }
    require(amount > 0 || _coll_withdrawl > 0 || _lusd_change > 0, "BorrowOperations: coll withdrawal and debt change must be greater than 0");
}

fn require_at_least_min_net_debt(_net_debt: u64) {
    require(_net_debt > MIN_NET_DEBT, "BorrowOperations: net debt must be greater than 0");
}

fn require_non_zero_debt_change(_debt_change: u64) {
    require(_debt_change > 0, "BorrowOperations: debt change must be greater than 0");
}

fn require_at_least_mcr(icr: u64) {
    require(icr > MCR, "Minimum collateral ratio not met");
}

fn require_valid_max_fee_percentage(_max_fee_percentage: u64) {
    require(_max_fee_percentage < DECIMAL_PRECISION, "BorrowOperations: max fee percentage must be less than 100");
}

#[storage(read)]
fn require_singular_coll_change(_coll_withdrawl: u64) {
    let amount = msg_amount();
    if amount > 0 {
        require_valid_asset_id();
    }
    require(_coll_withdrawl == 0 || 0 == amount, "BorrowOperations: collateral change must be 0 or equal to the amount sent");
}

#[storage(read)]
fn require_valid_asset_id() {
    require(msg_asset_id() == storage.asset_contract, "Invalid asset being transfered");
}

#[storage(read)]
fn withdraw_usdf(recipient: Identity, amount: u64, net_debt_increase: u64) {
    // increase the debt of the trove
    let usdf = abi(Token, storage.usdf_contract.value);
    usdf.mint_to_id(amount, recipient);
}

fn internal_get_coll_change(_coll_recieved: u64, _requested_coll_withdrawn: u64) -> (u64, bool) {
    if (_coll_recieved != 0) {
        return (_coll_recieved, true);
    } else {
        return (_requested_coll_withdrawn, false);
    }
}

fn internal_get_new_nomincal_icr_from_trove_change(
    _coll: u64,
    _debt: u64,
    _coll_change: u64,
    _is_coll_increase: bool,
    _debt_change: u64,
    _is_debt_increase: bool,
) -> u64 {
    let new_position = internal_get_new_trove_amounts(_coll, _debt, _coll_change, _is_coll_increase, _debt_change, _is_debt_increase);
    let new_icr = fm_compute_nominal_cr(new_position.0, new_position.1);

    return new_icr;
}

fn internal_get_new_trove_amounts(
    _coll: u64,
    _debt: u64,
    _coll_change: u64,
    _is_coll_increase: bool,
    _debt_change: u64,
    _is_debt_increase: bool,
) -> (u64, u64) {
    let new_coll = if _is_coll_increase {
        _coll + _coll_change
    } else {
        _coll - _coll_change
    };

    let new_debt = if _is_debt_increase {
        _debt + _debt_change
    } else {
        _debt - _debt_change
    };

    return (new_coll, new_debt);
}
