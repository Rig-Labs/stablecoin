library data_structures;

pub struct VestingSchedule {
    cliff_timestamp: u64,
    end_timestamp: u64,
    cliff_amount: u64,
    total_amount: u64,
    claimed_amount: u64,
    revocable: bool,
    recipient: Identity,
}

pub struct Asset {
    /// Identifier of asset
    id: ContractId,
    /// Amount of asset that can represent reserve amount, deposit amount, withdraw amount and more depending on the context
    amount: u64,
}

impl Asset {
    pub fn new(id: ContractId, amount: u64) -> Self {
        Self { id, amount }
    }
}
