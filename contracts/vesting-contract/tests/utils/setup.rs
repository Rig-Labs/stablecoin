use fuels::prelude::*;
use test_utils::setup::common::deploy_vesting_contract;
use test_utils::{interfaces::token::Token, setup::common::deploy_token};
// TODO: do setup instead of copy/pasted code with minor adjustments
use test_utils::interfaces::vesting::VestingContract;
// Load abi from json

pub async fn setup() -> (VestingContract, WalletUnlocked, WalletUnlocked, Token) {
    let config = Config {
        manual_blocks_enabled: true, // Necessary so the `produce_blocks` API can be used locally
        ..Config::local_node()
    };

    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        Some(config),
        None,
    )
    .await;

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let instance = deploy_vesting_contract(&wallet.clone()).await;

    let asset = deploy_token(&wallet).await;

    (instance, wallet, wallet2, asset)
}

pub mod test_helpers {

    use fuels::types::Identity;
    use test_utils::interfaces::token::{token_abi::initialize, token_abi::mint_to_id};

    use super::*;

    pub async fn init_and_mint_to_vesting(
        contract: &Token,
        vesting_contract: &VestingContract,
        amount: u64,
        admin: &WalletUnlocked,
    ) {
        let instance = Token::new(contract.id().clone(), admin.clone());
        let asset_id = AssetId::from(*instance.id().hash());
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

        let _ = mint_to_id(&instance, amount, admin).await;

        let _res = admin
            .force_transfer_to_contract(
                &vesting_contract.id(),
                amount,
                asset_id,
                TxParameters::default(),
            )
            .await;
    }
}
