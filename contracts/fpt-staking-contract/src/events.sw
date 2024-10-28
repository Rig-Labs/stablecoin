library;

pub struct StakeEvent {
    pub user: Identity,
    pub amount: u64,
}

pub struct UnstakeEvent {
    pub user: Identity,
    pub amount: u64,
}
