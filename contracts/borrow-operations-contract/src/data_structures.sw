library data_structures;

pub enum BorrowOperations {
    OpenTrove: (),
    CloseTrove: (),
    AdjustTrove: (),
}

pub struct LocalVariables_OpenTrove {
    price: u64,
    usdf_fee: u64,
    net_debt: u64,
    composite_debt: u64,
    icr: u64,
    nicr: u64,
    stake: u64,
    array_index: u64,
}

pub struct AssetContracts {
    active_pool: ContractId,
    coll_surplus_pool: ContractId,
    sorted_troves: ContractId,
    trove_manager: ContractId,
    oracle: ContractId,
}

impl LocalVariables_OpenTrove {
    pub fn new() -> Self {
        LocalVariables_OpenTrove {
            price: 0,
            usdf_fee: 0,
            net_debt: 0,
            composite_debt: 0,
            icr: 0,
            nicr: 0,
            stake: 0,
            array_index: 0,
        }
    }
}

pub struct LocalVariables_AdjustTrove {
    price: u64,
    coll_change: u64,
    net_debt_change: u64,
    is_coll_increase: bool,
    debt: u64,
    coll: u64,
    old_icr: u64,
    new_icr: u64,
    new_tcr: u64,
    usdf_fee: u64,
    new_debt: u64,
    new_coll: u64,
    stake: u64,
}

impl LocalVariables_AdjustTrove {
    pub fn new() -> Self {
        LocalVariables_AdjustTrove {
            price: 0,
            coll_change: 0,
            net_debt_change: 0,
            is_coll_increase: false,
            debt: 0,
            coll: 0,
            old_icr: 0,
            new_icr: 0,
            new_tcr: 0,
            usdf_fee: 0,
            new_debt: 0,
            new_coll: 0,
            stake: 0,
        }
    }
}
