use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "CollSurplusPool",
    abi = "contracts/coll-surplus-pool-contract/out/debug/coll-surplus-pool-contract-abi.json"
));

pub mod coll_surplus_pool_abi {
    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::active_pool::ActivePool;
    use fuels::prelude::Error;
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::types::AssetId;
    use fuels::{
        prelude::{Account, ContractId, TxPolicies, Wallet},
        types::Identity,
    };

    pub async fn initialize<T: Account + Clone>(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        borrow_operations: ContractId,
        protocol_manager: Identity,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = coll_surplus_pool
            .contract
            .methods()
            .initialize(borrow_operations, protocol_manager)
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn get_asset<T: Account + Clone>(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        asset: AssetId,
    ) -> Result<CallResponse<u64>, Error> {
        coll_surplus_pool
            .contract
            .methods()
            .get_asset(asset.into())
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn claim_coll<T: Account + Clone>(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        acount: Identity,
        active_pool: &ContractInstance<ActivePool<T>>,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        coll_surplus_pool
            .contract
            .methods()
            .claim_coll(acount, asset.into())
            .with_contracts(&[&active_pool.contract])
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
                active_pool.contract.contract_id().into(),
                active_pool.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await
    }

    pub async fn get_collateral(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<Wallet>>,
        acount: Identity,
        asset: AssetId,
    ) -> Result<CallResponse<u64>, Error> {
        coll_surplus_pool
            .contract
            .methods()
            .get_collateral(acount, asset.into())
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn add_asset<T: Account + Clone>(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        asset: AssetId,
        trove_manager: Identity,
    ) -> Result<CallResponse<()>, Error> {
        coll_surplus_pool
            .contract
            .methods()
            .add_asset(asset.into(), trove_manager)
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .call()
            .await
    }

    pub async fn account_surplus<T: Account + Clone>(
        coll_surplus_pool: &ContractInstance<CollSurplusPool<T>>,
        account: Identity,
        amount: u64,
        asset: AssetId,
    ) -> Result<CallResponse<()>, Error> {
        coll_surplus_pool
            .contract
            .methods()
            .account_surplus(account, amount, asset.into())
            .with_contract_ids(&[
                coll_surplus_pool.contract.contract_id().into(),
                coll_surplus_pool.implementation_id.into(),
            ])
            .call()
            .await
    }
}
