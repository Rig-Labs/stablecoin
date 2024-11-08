use fuels::prelude::{abigen, TxPolicies};
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "FPTStaking",
    abi = "contracts/fpt-staking-contract/out/debug/fpt-staking-contract-abi.json"
));

pub mod fpt_staking_abi {

    use super::*;
    use crate::data_structures::ContractInstance;
    use crate::interfaces::token::Token;
    use crate::interfaces::usdf_token::USDFToken;
    use fuels::prelude::{Account, AssetId, CallParameters, Error};
    use fuels::types::transaction_builders::VariableOutputPolicy;
    use fuels::{prelude::ContractId, types::Identity};

    pub async fn initialize<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        protocol_manager_address: ContractId,
        borrower_operations_address: ContractId,
        fpt_asset_id: AssetId,
        usdf_asset_id: AssetId,
    ) -> CallResponse<()> {
        let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .initialize(
                protocol_manager_address,
                borrower_operations_address,
                fpt_asset_id.into(),
                usdf_asset_id.into(),
            )
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_storage<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
    ) -> CallResponse<fpt_staking_abi::ReadStorage> {
        let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .get_storage()
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn stake<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        fpt_asset_id: AssetId,
        fpt_deposit_amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_script_gas_limit(2000000);

        let call_params: CallParameters = CallParameters::default()
            .with_amount(fpt_deposit_amount)
            .with_asset_id(fpt_asset_id);

        fpt_staking
            .contract
            .methods()
            .stake()
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .with_tx_policies(tx_params)
            .call_params(call_params)
            .unwrap()
            .call()
            .await
    }

    pub async fn unstake<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        usdf_token: &ContractInstance<USDFToken<T>>,
        mock_token: &Token<T>,
        fpt_token: &Token<T>,
        amount: u64,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default()
            .with_tip(1)
            .with_witness_limit(2000000)
            .with_script_gas_limit(2000000);

        fpt_staking
            .contract
            .methods()
            .unstake(amount)
            .with_tx_policies(tx_params)
            .with_contracts(&[&usdf_token.contract, mock_token, fpt_token])
            .with_contract_ids(&[
                usdf_token.contract.contract_id().into(),
                usdf_token.implementation_id.into(),
                mock_token.contract_id().into(),
                fpt_token.contract_id().into(),
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .with_variable_output_policy(VariableOutputPolicy::Exactly(10))
            .call()
            .await
    }

    pub async fn add_asset<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        asset_address: AssetId,
    ) -> CallResponse<()> {
        // let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .add_asset(asset_address.into())
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_asset_gain<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        id: Identity,
        asset_address: AssetId,
    ) -> CallResponse<u64> {
        // let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .get_pending_asset_gain(id, asset_address.into())
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_usdf_gain<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        id: Identity,
    ) -> CallResponse<u64> {
        // let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .get_pending_usdf_gain(id)
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_f_usdf<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        usdf_fee_amount: u64,
    ) -> CallResponse<()> {
        // let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .increase_f_usdf(usdf_fee_amount)
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_f_asset<T: Account>(
        fpt_staking: &ContractInstance<FPTStaking<T>>,
        asset_fee_amount: u64,
        asset_address: AssetId,
    ) -> CallResponse<()> {
        // let tx_params = TxPolicies::default().with_tip(1);

        fpt_staking
            .contract
            .methods()
            .increase_f_asset(asset_fee_amount, asset_address.into())
            .with_contract_ids(&[
                fpt_staking.contract.contract_id().into(),
                fpt_staking.implementation_id.into(),
            ])
            .call()
            .await
            .unwrap()
    }
}
