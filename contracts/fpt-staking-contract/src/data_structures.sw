library data_structures;

pub struct ReadStorage {
    f_usdf: u64,
    total_fpt_staked: u64,
    protocol_manager_address: ContractId,
    borrower_operations_address: ContractId,
    fpt_address: ContractId,
    usdf_address: ContractId,
    is_initialized: bool,
}
