use fuels::{prelude::abigen, programs::responses::CallResponse};

abigen!(Contract(
    name = "FPTToken",
    abi = "contracts/fpt-token-contract/out/debug/fpt-token-contract-abi.json"
));

pub mod fpt_token_abi {
    use crate::interfaces::vesting::VestingContract;
    use crate::{
        data_structures::ContractInstance, interfaces::community_issuance::CommunityIssuance,
    };
    use fuels::{prelude::*, types::ContractId};

    use super::*;
    pub async fn initialize<T: Account>(
        instance: &FPTToken<T>,
        vesting_contract: &VestingContract<T>,
        community_issuance_contract: &ContractInstance<CommunityIssuance<T>>,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        let res = instance
            .methods()
            .initialize(
                vesting_contract.contract_id(),
                community_issuance_contract.contract.contract_id(),
            )
            .with_contracts(&[vesting_contract, &community_issuance_contract.contract])
            .with_contract_ids(&[
                community_issuance_contract.contract.contract_id().into(),
                community_issuance_contract.implementation_id.into(),
                vesting_contract.contract_id().into(),
            ])
            .with_tx_policies(tx_params)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(10))
            .call()
            .await;

        return res.unwrap();
    }

    pub async fn total_supply<T: Account>(instance: &FPTToken<T>) -> CallResponse<Option<u64>> {
        let fpt_token_asset_id = instance
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        instance
            .methods()
            .total_supply(fpt_token_asset_id)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_vesting_contract<T: Account>(
        instance: &FPTToken<T>,
    ) -> CallResponse<ContractId> {
        instance
            .methods()
            .get_vesting_contract()
            .call()
            .await
            .unwrap()
    }
}
