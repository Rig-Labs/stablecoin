use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "ProtocolManager",
    abi = "contracts/protocol-manager-contract/out/debug/protocol-manager-contract-abi.json"
));

pub mod protocol_manager_abi {
    use crate::interfaces::borrow_operations::BorrowOperations;
    use crate::interfaces::stability_pool::StabilityPool;
    use crate::interfaces::usdf_token::USDFToken;
    use crate::setup::common::{wait, AssetContracts};
    use fuels::prelude::{Account, CallParameters, LogDecoder, SettableContract};
    use fuels::types::AssetId;
    use fuels::{
        prelude::{ContractId, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        borrow_operations: ContractId,
        stability_pool: ContractId,
        usdf: ContractId,
        admin: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = protocol_manager
            .methods()
            .initialize(borrow_operations, stability_pool, usdf, admin)
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

    pub async fn register_asset<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        asset: ContractId,
        active_pool: ContractId,
        trove_manager: ContractId,
        coll_surplus_pool: ContractId,
        oracle: ContractId,
        sorted_troves: ContractId,
        borrow_operations: &BorrowOperations<T>,
        stability_pool: &StabilityPool<T>,
        usdf: &USDFToken<T>,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        protocol_manager
            .methods()
            .register_asset(
                asset,
                active_pool,
                trove_manager,
                coll_surplus_pool,
                oracle,
                sorted_troves,
            )
            .tx_params(tx_params)
            .set_contracts(&[borrow_operations, stability_pool, usdf])
            .call()
            .await
            .unwrap()
    }

    pub async fn redeem_collateral<T: Account>(
        protocol_manager: &ProtocolManager<T>,
        amount: u64,
        max_iterations: u64,
        max_fee_percentage: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Option<Identity>,
        lower_partial_hint: Option<Identity>,
        usdf: &USDFToken<T>,
        asset_contracts: &Vec<AssetContracts<T>>,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default()
            .set_gas_price(1)
            .set_gas_limit(2000000);
        let usdf_asset_id = AssetId::from(*usdf.contract_id().hash());

        let call_params: CallParameters = CallParameters::default()
            .set_amount(amount)
            .set_asset_id(usdf_asset_id);

        let mut set_contracts: Vec<&dyn SettableContract> = Vec::new();

        for contracts in asset_contracts.iter() {
            set_contracts.push(&contracts.active_pool);
            set_contracts.push(&contracts.trove_manager);
            set_contracts.push(&contracts.coll_surplus_pool);
            set_contracts.push(&contracts.oracle);
            set_contracts.push(&contracts.sorted_troves);
            set_contracts.push(&contracts.default_pool)
        }

        set_contracts.push(usdf);

        protocol_manager
            .methods()
            .redeem_collateral(
                max_iterations,
                max_fee_percentage,
                partial_redemption_hint,
                upper_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
                lower_partial_hint.unwrap_or(Identity::Address([0; 32].into())),
            )
            .tx_params(tx_params)
            .call_params(call_params)
            .unwrap()
            .set_contracts(&set_contracts)
            .append_variable_outputs(10)
            .call()
            .await
            .unwrap()
    }
}
