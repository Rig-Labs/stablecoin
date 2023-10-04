use fuels::prelude::abigen;
use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "CommunityIssuance",
    abi = "contracts/community-issuance-contract/out/debug/community-issuance-contract-abi.json"
));

pub mod community_issuance_abi {
    use fuels::prelude::{Account, LogDecoder, TxParameters};

    use crate::setup::common::wait;
    use fuels::{prelude::ContractId, prelude::Error, types::AssetId, types::Identity};

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &CommunityIssuance<T>,
        stability_pool_contract: ContractId,
        fpt_token_asset_id: AssetId,
        admin: &Identity,
        debugging: bool,
    ) -> Result<FuelCallResponse<()>, Error> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = instance
            .methods()
            .initialize(
                stability_pool_contract,
                fpt_token_asset_id.into(),
                admin.clone(),
                debugging,
            )
            .tx_params(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn set_current_time<T: Account>(
        instance: &CommunityIssuance<T>,
        time: u64,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = instance
            .methods()
            .set_current_time(time)
            .tx_params(tx_params)
            .call()
            .await;

        return res.unwrap();
    }

    pub async fn public_start_rewards_increase_transition_after_deadline<T: Account>(
        instance: &CommunityIssuance<T>,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = instance
            .methods()
            .public_start_rewards_increase_transition_after_deadline()
            .tx_params(tx_params)
            .call()
            .await;

        return res.unwrap();
    }

    pub async fn start_rewards_increase_transition<T: Account>(
        instance: &CommunityIssuance<T>,
        transition_time: u64,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().with_gas_price(1);

        let res = instance
            .methods()
            .start_rewards_increase_transition(transition_time)
            .tx_params(tx_params)
            .call()
            .await;

        return res.unwrap();
    }
}
