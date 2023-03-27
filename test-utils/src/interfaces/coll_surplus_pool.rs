use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod coll_surplus_pool_abi {
    use crate::{interfaces::active_pool::ActivePool, setup::common::wait};
    use fuels::{
        prelude::{ContractId, LogDecoder, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize(
        default_pool: &CollSurplusPool,
        trove_manager: Identity,
        active_pool: ContractId,
        borrow_operations: ContractId,
        asset_id: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = default_pool
            .methods()
            .initialize(
                trove_manager.clone(),
                active_pool,
                borrow_operations,
                asset_id,
            )
            .tx_params(tx_params)
            .call()
            .await;

        // TODO: remove this workaround
        match res {
            Ok(res) => res,
            Err(_) => {
                wait();
                return FuelCallResponse::new((), vec![], LogDecoder::default());
            }
        }
    }

    pub async fn get_asset(
        default_pool: &CollSurplusPool,
        asset: ContractId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_asset(asset)
            .call()
            .await
            .unwrap()
    }

    pub async fn claim_coll(
        default_pool: &CollSurplusPool,
        acount: Identity,
        active_pool: &ActivePool,
        asset: ContractId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .claim_coll(acount, asset)
            .set_contracts(&[active_pool])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_collateral(
        default_pool: &CollSurplusPool,
        acount: Identity,
        asset: ContractId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_collateral(acount, asset)
            .call()
            .await
            .unwrap()
    }
}
