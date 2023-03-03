library interface;

dep data_structures;
dep sorted_troves_interface;
dep trove_manager_interface;
dep borrow_operations_interface;
dep fluid_math;
dep token_interface;
dep active_pool_interface;
dep stability_pool_interface;

use data_structures::{Trove};

abi MockOracle {
    #[storage(write)]
    fn set_price(price: u64);

    #[storage(read)]
    fn get_price() -> u64;

    #[storage(read)]
    fn get_precision() -> u64;
}
