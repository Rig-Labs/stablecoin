use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod coll_surplus_pool_abi {
    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use fuels::prelude::Error;
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::types::AssetId;
    use fuels::{
        prelude::{Account, ContractId, TxPolicies, WalletUnlocked},
        types::Identity,
    };

    pub async fn initialize<T: Account>(
        coll_surplus_pool: &CollSurplusPool<T>,
        borrow_operations: ContractId,
        protocol_manager: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

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
    ) -> Result<CallResponse<u64>, Error> {
        default_pool.methods().get_asset(asset.into()).call().await
    }

    pub async fn claim_coll<T: Account>(
        default_pool: &CollSurplusPool<T>,
        acount: Identity,
        active_pool: &ActivePool<T>,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        default_pool
            .methods()
            .claim_coll(acount, asset.into())
            .with_contracts(&[active_pool])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }

    pub async fn get_collateral(
        default_pool: &CollSurplusPool<WalletUnlocked>,
        acount: Identity,
        asset: AssetId,
    ) -> Result<CallResponse<u64>, Error> {
        default_pool
            .methods()
            .get_collateral(acount, asset.into())
            .call()
            .await
    }

    pub async fn add_asset<T: Account>(
        default_pool: &CollSurplusPool<T>,
        asset: AssetId,
        trove_manager: Identity,
    ) -> Result<CallResponse<()>, Error> {
        default_pool
            .methods()
            .add_asset(asset.into(), trove_manager)
            .call()
            .await
    }

    pub async fn account_surplus<T: Account>(
        default_pool: &CollSurplusPool<T>,
        account: Identity,
        amount: u64,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        default_pool
            .methods()
            .account_surplus(account, amount, asset.into())
            .call()
            .await
    }
}
