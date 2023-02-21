library data_structures;

pub struct Trove {
    usdf_borrowed: u64,
    fuel_deposited: u64,
    st_fuel_deposited: u64,
}

pub struct Asset {
    /// Identifier of asset
    id: ContractId,
    /// Amount of asset that can represent reserve amount, deposit amount, withdraw amount and more depending on the context
    amount: u64,
}

pub struct Node {
    exists: bool,
    next_id: Identity,
    prev_id: Identity,
}

impl Asset {
    pub fn new(id: ContractId, amount: u64) -> Self {
        Self { id, amount }
    }
}
