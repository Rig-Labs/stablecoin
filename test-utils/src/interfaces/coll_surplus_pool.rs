use fuels::prelude::abigen;
use fuels::programs::call_response::FuelCallResponse;
use fuels::programs::call_utils::TxDependencyExtension;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod coll_surplus_pool_abi {
    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use fuels::prelude::Error;
    use fuels::types::AssetId;
    use fuels::{
        prelude::{Account, ContractId, TxPolicies, WalletUnlocked},
        types::Identity,
    };

    pub async fn initialize<T: Account>(
        coll_surplus_pool: &CollSurplusPool<T>,
        borrow_operations: ContractId,
        protocol_manager: Identity,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_gas_price(1);

        let res = coll_surplus_pool
            .methods()
            .initialize(borrow_operations, protocol_manager)
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_asset<T: Account>(
        default_pool: &CollSurplusPool<T>,
        asset: AssetId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_asset(asset.into())
            .call()
            .await
            .unwrap()
    }

    pub async fn claim_coll<T: Account>(
        default_pool: CollSurplusPool<T>,
        acount: Identity,
        active_pool: &ActivePool<T>,
        asset: AssetId,
    ) -> FuelCallResponse<()> {
        default_pool
            .methods()
            .claim_coll(acount, asset.into())
            .with_contracts(&[active_pool])
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_collateral(
        default_pool: &CollSurplusPool<WalletUnlocked>,
        acount: Identity,
        asset: AssetId,
    ) -> FuelCallResponse<u64> {
        default_pool
            .methods()
            .get_collateral(acount, asset.into())
            .call()
            .await
            .unwrap()
    }
}
