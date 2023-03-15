use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod default_pool_abi {
    use crate::interfaces::active_pool::ActivePool;
    use fuels::{prelude::ContractId, types::Identity};

    use super::*;

    pub async fn initialize(
        default_pool: &CollSurplusPool,
        trove_manager: Identity,
        active_pool: ContractId,
        borrow_operations: ContractId,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .initialize(trove_manager, active_pool, borrow_operations, asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_asset(default_pool: &CollSurplusPool) -> FuelCallResponse<u64> {
        default_pool.methods().get_asset().call().await.unwrap()
    }

    pub async fn claim_coll(
        default_pool: &CollSurplusPool,
        acount: Identity,
        active_pool: &ActivePool,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .claim_coll(acount)
            .set_contracts(&[active_pool])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_collateral(
        default_pool: &CollSurplusPool,
        acount: Identity,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_collateral(acount)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }
}
