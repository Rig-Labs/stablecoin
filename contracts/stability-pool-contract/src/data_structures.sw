library;

use libraries::fluid_math::numbers::*;
use std::u128::U128;
pub struct
 AssetContracts {
    pub trove_manager: ContractId,
    pub oracle: ContractId,
}
pub struct
 Snapshots {
    pub P: U128,
    pub G: U128,
    pub scale: u64,
    pub epoch: u64,
}
impl Snapshots
 {
    pub fn default() -> Self {
        Snapshots {
            P: U128::from_u64(0),
            G: U128::from_u64(0),
            scale: 0,
            epoch: 0,
        }
    }
}
