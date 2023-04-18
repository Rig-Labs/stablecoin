use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod coll_surplus_pool_abi {
    use crate::{interfaces::active_pool::ActivePool, setup::common::wait};
    use fuels::{
        prelude::{Account, ContractId, LogDecoder, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize<T: Account>(
        default_pool: &CollSurplusPool<T>,
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

    pub async fn get_asset<T: Account>(default_pool: CollSurplusPool<T>) -> FuelCallResponse<u64> {
        default_pool.methods().get_asset().call().await.unwrap()
    }

    pub async fn claim_coll<T: Account>(
        default_pool: CollSurplusPool<T>,
        acount: Identity,
        active_pool: &ActivePool<T>,
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

    pub async fn get_collateral<T: Account>(
        default_pool: CollSurplusPool<T>,
        acount: Identity,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_collateral(acount)
            .call()
            .await
            .unwrap()
    }
}
