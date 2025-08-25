use fuels::prelude::*;
use test_utils::data_structures::ContractInstance;
use test_utils::interfaces::vesting::VestingContract;
use test_utils::setup::common::deploy_vesting_contract;
use test_utils::{interfaces::token::Token, setup::common::deploy_token};

pub async fn setup(
    total_amount: u64,
) -> (
    ContractInstance<VestingContract<Wallet>>,
    Wallet,
    Wallet,
    Token<Wallet>,
) {
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    // First deploy the target contract
    let vesting = deploy_vesting_contract(&wallet.clone(), total_amount).await;

    let asset = deploy_token(&wallet).await;

    (vesting, wallet, wallet2, asset)
}

pub mod test_helpers {

    use fuels::types::Identity;
    use test_utils::interfaces::token::{token_abi::initialize, token_abi::mint_to_id};

    use super::*;

    pub async fn init_and_mint_to_vesting(
        contract: &Token<Wallet>,
        vesting_contract: &VestingContract<Wallet>,
        amount: u64,
        admin: &Wallet,
    ) {
        let instance = Token::new(contract.contract_id().clone(), admin.clone());
        let asset_id = instance
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();
        let name = "Fluid Protocol Test Token".to_string();
        let symbol = "FPTT".to_string();

        let _ = initialize(
            &instance,
            amount,
            &Identity::Address(admin.address().into()),
            name,
            symbol,
        )
        .await;

        let _ = mint_to_id(&instance, amount, Identity::Address(admin.address().into())).await;

        let _res = admin
            .force_transfer_to_contract(
                &vesting_contract.contract_id(),
                amount,
                asset_id,
                TxPolicies::default(),
            )
            .await
            .unwrap();
    }
}
