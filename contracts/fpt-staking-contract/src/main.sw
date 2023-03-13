contract;

storage {
    stakes: StorageVec<Identity, u64> = StorageVec {},
    snapshots: StorageVec<Identity, Snapshot> = StorageVec {},
    f_fuel: u64 = 0,
    f_usdf: u64 = 0,
    total_fpt_staked: u64 = 0
}

const DECIMAL_PRECISION = 1; //temporary until we figure this out!

impl FPTStaking for Contract {
    #[storage(read, write)]
    fn stake(id: Identity, amount: u64) -> u64 {
        require_non_zero(amount);
        let current_stake = storage.stakes.get(id);
        let mut fuel_gain = 0;
        let mut usdf_gain = 0;
        if (current_stake != 0){
            fuel_gain = get_pending_fuel_gain(id);
            usdf_gain = get_pending_usdf_gain(id);
        }
        update_user_snapshots(id);


    }

    #[storage(read)]
    fn unstake(id: Identity, amount: u64) -> u64 {

    }
}

// doesn't really look like in Sway we need to have a public wrapper of a private function for these 
#[storage(read)]
fn get_pending_fuel_gain(id: Identity) -> u64 {
    let f_fuel_snapshot = storage.snapshots.get(id).f_fuel_snapshot;
    let fuel_gain = (storage.stakes(id) * (storage.f_fuel - f_fuel_snapshot)) / DECIMAL_PRECISION;
    fuel_gain
}

#[storage(read)]
fn get_pending_usdf_gain(id: Identity) ->  u64 {
    let f_usdf_snapshot = storage.snapshots.get(id).f_usdf_snapshot;
    let usdf_gain = (storage.stakes(id) * (storage.f_usdf - f_usdf_snapshot)) / DECIMAL_PRECISION;
    usdf_gain
}

// is this model going to work with parallel tx's in fuel? In general do we need to be worried about race conditions *during* tx's?
#[storage(read, write)]
fn update_user_snapshots(id: Identity) -> {
    let mut user_snapshot = storage.snapshots.get(id);
    user_snapshot.f_usdf_snapshot = storage.f_usd;
    user_snapshot.f_fuel_snapshot = storage.f_fuel;
    storage.snapshots.insert(id, user_snapshot);
}

fn require_non_zero(amount: u64) {
    require(amount > 0, "Amount must be greater than 0");
}
