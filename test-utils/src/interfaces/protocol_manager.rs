use fuels::prelude::abigen;
use fuels::programs::call_response::FuelCallResponse;
use fuels::programs::call_utils::TxDependencyExtension;

abigen!(Contract(
    name = "ProtocolManager",
    abi = "contracts/protocol-manager-contract/out/debug/protocol-manager-contract-abi.json"
));

pub mod protocol_manager_abi {
    use super::*;
    use crate::interfaces::active_pool::ActivePool;
    use crate::interfaces::borrow_operations::BorrowOperations;
    use crate::interfaces::coll_surplus_pool::CollSurplusPool;
    use crate::interfaces::default_pool::DefaultPool;
    use crate::interfaces::fpt_staking::FPTStaking;
    use crate::interfaces::sorted_troves::SortedTroves;
    use crate::interfaces::stability_pool::StabilityPool;
    use crate::interfaces::usdf_token::USDFToken;
    use crate::setup::common::AssetContracts;
    use fuels::prelude::{Account, CallParameters, SettableContract};
    use fuels::types::AssetId;
    use fuels::{
        prelude::{ContractId, TxParameters},
        types::Identity,
    };

    pub async fn initialize<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        borrow_operations: ContractId,
        stability_pool: ContractId,
        fpt_staking: ContractId,
        usdf: ContractId,
        coll_surplus_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        sorted_troves: ContractId,
        admin: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = protocol_manager
            .methods()
            .initialize(
                borrow_operations,
                stability_pool,
                fpt_staking,
                usdf,
                coll_surplus_pool,
                default_pool,
                active_pool,
                sorted_troves,
                admin,
            )
            .tx_params(tx_params)
            .call()
            .await
            .unwrap();

        return res;

        // TODO: remove this workaround
        // match res {
        //     Ok(res) => res,
        //     Err(_) => {
        //         wait();
        //         return FuelCallResponse::new((), vec![], LogDecoder::default());
        //     }
        // }
    }

    pub async fn register_asset<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        asset: ContractId,
        trove_manager: ContractId,
        oracle: ContractId,
        borrow_operations: &BorrowOperations<T>,
        stability_pool: &StabilityPool<T>,
        usdf: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        coll_surplus_pool: &CollSurplusPool<T>,
        default_pool: &DefaultPool<T>,
        active_pool: &ActivePool<T>,
        sorted_troves: &SortedTroves<T>,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        protocol_manager
            .methods()
            .register_asset(asset, trove_manager, oracle)
            .tx_params(tx_params)
            .with_contracts(&[
                borrow_operations,
                stability_pool,
                usdf,
                fpt_staking,
                coll_surplus_pool,
                default_pool,
                active_pool,
                sorted_troves,
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn redeem_collateral<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        amount: u64,
        max_iterations: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Option<Identity>,
        lower_partial_hint: Option<Identity>,
        usdf: &USDFToken<T>,
        fpt_staking: &FPTStaking<T>,
        coll_surplus_pool: &CollSurplusPool<T>,
        default_pool: &DefaultPool<T>,
        active_pool: &ActivePool<T>,
        sorted_troves: &SortedTroves<T>,
        aswith_contracts: &Vec<AssetContracts<T>>,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default()
            .with_gas_price(1)
            .with_gas_limit(2000000);
        let usdf_asset_id = AssetId::from(*usdf.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .with_amount(amount)
            .with_asset_id(usdf_asset_id);

        let mut with_contracts: Vec<&dyn SettableContract> = Vec::new();

        for contracts in aswith_contracts.iter() {
            with_contracts.push(&contracts.trove_manager);
            with_contracts.push(&contracts.oracle);
        }

        with_contracts.push(fpt_staking);
        with_contracts.push(coll_surplus_pool);
        with_contracts.push(default_pool);
        with_contracts.push(active_pool);
        with_contracts.push(usdf);
        with_contracts.push(sorted_troves);

        protocol_manager
            .methods()
            .redeem_collateral(
                max_iterations,
                partial_redemption_hint,
                upper_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
                lower_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
            )
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .with_contracts(&with_contracts)
            .append_variable_outputs(10)
            .call()
            .await
            .unwrap()
    }
}
