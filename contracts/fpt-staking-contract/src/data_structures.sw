library data_structures;

pub struct Snapshot {
    f_fuel_snapshot: u64,
    f_usdf_snapshot: u64,
}

impl Snapshot {
    pub fn default() self -> {
        Snapshot {
            f_fuel_snapshot: 0,
            f_usdf_snapshot: 0,
        }
    }
}