contract;

use interface::FluidProtocol;
// based on https://github.com/liquity/dev/blob/main/packages/contracts/contracts/Dependencies/LiquityBase.sol

use active_pool::ActivePool;
use default_pool::DefaultPool;
use utils::compute_CR;

impl FluidProtocol for Contract {

    //todo: convert decimal places and math to u64

    const 100_PCT = 1000000000000000000;
    const CR = 1350000000000000000;
    const USDF_GAS_COMPENSATION = 200e18; // not sure if e18 works in sway
    const MIN_NET_DEBT = 1000e18; // not sure if e18 works in sway
    const PERCENT_DIVISOR = 200;
    const ACTIVE_POOL_CONTRACT_ID = 0x0;
    const DEFAULT_POOL_CONTRACT_ID = 0x0;
    const DECIMAL_PRECISION = 1e18;
    const BORROWING_FEE_FLOOR = DECIMAL_PRECISION / 1000 * 5; // 0.5%

    // Not sure where these functions are used yet...
    // Should we move (some of?) these to utils.sw?
    // on the other hand, these functions all rely on the protocol constants defined here.
    // note: all of these function are pure, they do not access storage.

    function get_coll_gas_compensation(entire_coll: u64) -> u64 {
        entire_coll / PERCENT_DIVISOR
    }

    function get_entire_system_coll() -> u64 {
        let active_pool = abi(ActivePool, ACTIVE_POOL_CONTRACT_ID);
        let default_pool = abi(DefaultPool, DEFAULT_POOL_CONTRACT_ID);
        let active_coll = active_pool.get_fuel_coll();
        let liquidated_coll = default_pool.get_fuel_coll();

        active_coll + liquidated_coll // sway has safe math by default
    }

    function get_entire_system_debt() -> u64 {
        let active_pool = abi(ActivePool, ACTIVE_POOL_CONTRACT_ID);
        let default_pool = abi(DefaultPool, DEFAULT_POOL_CONTRACT_ID);
        let active_debt = active_pool.get_usdf_debt();
        let closed_debt = default_pool.get_usdf_debt();

        active_debt + closed_debt
    }

    function get_TCR(price: u64) -> u64 {
        let entire_system_coll = get_entire_system_coll();
        let entire_system_debt = get_entire_system_debt();

        compute_CR(entire_system_coll, entire_system_debt, price)
    }

    function check_TCR(price: u64) -> bool {
        let TCR = get_TCR(price);
        TCR < CR
    }

    function require_user_accepts_fee(fee: u64, amount: u64, max_fee_percentage: u64){
        let fee_percentage = (fee * DECIMAL_PRECISION) / amount;
        log("checking if fee exceeded provided maximum..."); // just going to make a log before every assert for now, since assert doesn't have an error message    
        assert(fee_percentage < max_fee_percentage);
    }

}
