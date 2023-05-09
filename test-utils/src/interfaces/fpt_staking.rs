use fuels::prelude::{abigen, TxParameters};

use fuels::programs::call_response::FuelCallResponse;

abigen!(Contract(
    name = "FPTStaking",
    abi = "contracts/fpt-staking-contract/out/debug/fpt-staking-contract-abi.json"
));

pub mod fpt_staking_abi {

    use fuels::client::schema::schema::__fields::Contract;
    use fuels::prelude::{Account};
    use fuels::{
        prelude::{ContractId},
        types::Identity,
    };
    // use crate::setup::common::wait;

    use super::*;

    pub async fn initialize<T: Account>(
        fpt_staking: &FPTStaking<T>,
        protocol_manager_address: ContractId,
        trove_manager_address: ContractId,
        borrower_operations_address: ContractId,
        fpt_address: ContractId,
        usdf_address: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .initialize(
                protocol_manager_address,
                trove_manager_address,
                borrower_operations_address,
                fpt_address,
                usdf_address,
            )
            .call()
            .await
            .unwrap()
    }

    pub async fn stake<T: Account>(
        fpt_staking: &FPTStaking<T>,
        id: Identity,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .stake(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn unstake<T: Account>(
        fpt_staking: &FPTStaking<T>,
        id: Identity,
        amount: u64,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .unstake(id, amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn add_asset<T: Account>(
        fpt_staking: &FPTStaking<T>,
        trove_manager_address: ContractId,
        active_pool_address: ContractId,
        sorted_troves_address: ContractId,
        asset_address: ContractId,
        oracle_address: ContractId,
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .add_asset(trove_manager_address,
                active_pool_address,
                sorted_troves_address,
                asset_address,
                oracle_address
            )
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_asset_gain<T: Account>(
        fpt_staking: &FPTStaking<T>,
        id: Identity,
        asset_address: ContractId
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .get_pending_asset_gain(id, asset_address)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_pending_usdf_gain<T: Account>(
        fpt_staking: &FPTStaking<T>,
        id: Identity
    ) -> FuelCallResponse<u64> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .get_pending_usdf_gain(id)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_f_usdf<T: Account>(
        fpt_staking: &FPTStaking<T>,
        usdf_fee_amount: u64
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .increase_f_usdf(usdf_fee_amount)
            .call()
            .await
            .unwrap()
    }

    pub async fn increase_f_asset<T: Account>(
        fpt_staking: &FPTStaking<T>,
        asset_fee_amount: u64, 
        asset_address: ContractId
    ) -> FuelCallResponse<()> {
        let tx_params = TxParameters::default().set_gas_price(1);

        fpt_staking
            .methods()
            .increase_f_asset(asset_fee_amount, asset_address)
            .call()
            .await
            .unwrap()
    }

}