use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "CommunityIssuance",
    abi = "contracts/community-issuance-contract/out/debug/community-issuance-contract-abi.json"
));

pub mod community_issuance_abi {
    use fuels::prelude::{Account, TxPolicies};

    use fuels::{prelude::ContractId, prelude::Error, types::AssetId, types::Identity};

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &CommunityIssuance<T>,
        stability_pool_contract: ContractId,
        fpt_token_asset_id: AssetId,
        admin: &Identity,
        debugging: bool,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = instance
            .methods()
            .initialize(
                stability_pool_contract,
                fpt_token_asset_id.into(),
                admin.clone(),
                debugging,
            )
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }

    pub async fn set_current_time<T: Account>(
        instance: &CommunityIssuance<T>,
        time: u64,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = instance
            .methods()
            .set_current_time(time)
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res.unwrap();
    }

    pub async fn public_start_rewards_increase_transition_after_deadline<T: Account>(
        instance: &CommunityIssuance<T>,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = instance
            .methods()
            .public_start_rewards_increase_transition_after_deadline()
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res.unwrap();
    }

    pub async fn start_rewards_increase_transition<T: Account>(
        instance: &CommunityIssuance<T>,
        transition_time: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = instance
            .methods()
            .start_rewards_increase_transition(transition_time)
            .with_tx_policies(tx_params)
            .call()
            .await;

        return res;
    }
}
