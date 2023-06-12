use fuels::{prelude::abigen, programs::call_response::FuelCallResponse, types::Identity};

abigen!(Contract(
    name = "CommunityIssuance",
    abi = "contracts/community-issuance-contract/out/debug/community-issuance-contract-abi.json"
));

pub mod community_issuance_abi {
    use fuels::{prelude::{Account, LogDecoder, TxParameters}, accounts::fuel_crypto::coins_bip32::ecdsa::digest::typenum::U256};

    use crate::setup::common::wait;
    use fuels::{
        prelude::{ContractId},
        types::Identity,
    };

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &CommunityIssuance<T>,
        stability_pool_contract: ContractId,
        fpt_token_contract: ContractId,
        admin: Identity,
        debugging: bool,
        time: u64,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        let res = instance
            .methods()
            .initialize(stability_pool_contract, fpt_token_contract, admin, debugging, time)
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

    pub async fn get_cumulative_issuance_fraction<T: Account>(
        community_issuance: &CommunityIssuance<T>,
        x: u64,
        y: u64
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::default().set_gas_price(1).set_gas_limit(20000000);

        community_issuance
            .methods()
            .get_cumulative_issuance_fraction(x, y)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn external_test_issue_fpt<T: Account>(
        community_issuance: &CommunityIssuance<T>,
        current_time: u64, 
        deployment_time: u64, 
        time_transition_started: u64, 
        total_transition_time_seconds:u64, 
        total_fpt_issued: u64, 
        has_transitioned_rewards:bool
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::default().set_gas_price(1).set_gas_limit(20000000);

        community_issuance
            .methods()
            .external_test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }
}
