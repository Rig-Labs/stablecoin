library data_structures;

pub struct Snapshots {
    S: u64,
    P: u64,
    G: u64,
    scale: u64,
    epoch: u64,
}

impl Snapshots {
    pub fn default() -> Self {
        Snapshots {
            S: 0,
            P: 0,
            G: 0,
            scale: 0,
            epoch: 0,
        }
    }
}
// TODO Change to u128 when possible
