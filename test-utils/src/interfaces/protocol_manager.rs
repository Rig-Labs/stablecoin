use fuels::prelude::abigen;

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "ProtocolManager",
    abi = "contracts/protocol-manager-contract/out/debug/protocol-manager-contract-abi.json"
));

pub mod protocol_manager_abi {
    use crate::interfaces::borrow_operations::BorrowOperations;
    use crate::interfaces::stability_pool::StabilityPool;
    use crate::setup::common::wait;
    use fuels::prelude::LogDecoder;
    use fuels::{
        prelude::{ContractId, TxParameters},
        types::Identity,
    };

    use super::*;

    pub async fn initialize(
        protocol_manager: &ProtocolManager,
        borrow_operations: ContractId,
        stability_pool: ContractId,
        admin: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = protocol_manager
            .methods()
            .initialize(borrow_operations, stability_pool, admin)
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

    pub async fn register_asset(
        protocol_manager: &ProtocolManager,
        asset: ContractId,
        active_pool: ContractId,
        trove_manager: ContractId,
        coll_surplus_pool: ContractId,
        oracle: ContractId,
        sorted_troves: ContractId,
        borrow_operations: &BorrowOperations,
        stability_pool: &StabilityPool,
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
            .set_contracts(&[borrow_operations, stability_pool])
            .call()
            .await
            .unwrap()
    }
}
