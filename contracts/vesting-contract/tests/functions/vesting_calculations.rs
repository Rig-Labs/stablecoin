use crate::utils::setup::setup;
use fuels::prelude::*;
use fuels::{prelude::AssetId, types::Identity};

mod success {
    use crate::utils::setup::test_helpers::{
        get_vesting_schedule, instantiate_vesting_contract, mint_to_vesting,
    };

    use super::*;

    #[tokio::test]
    async fn create_vesting_contract() {
        let (vest, admin, recipient, asset) = setup().await;

        let vesting_schedule = [get_vesting_schedule(
            3000,
            1000,
            1000,
            0,
            10000,
            Identity::Address(recipient.address().into()),
            false,
        )];

        let _ = instantiate_vesting_contract(
            &vest,
            &admin.address().into(),
            &vesting_schedule.to_vec(),
            &asset,
            10000,
        )
        .await;

        let res = vest
            .methods()
            .get_vesting_schedule(Identity::Address(recipient.address().into()))
            .call()
            .await
            .unwrap();

        assert_eq!(res.value.unwrap(), vesting_schedule[0]);

        let res = vest
            .methods()
            .get_vesting_schedule(Identity::Address(admin.address().into()))
            .call()
            .await
            .unwrap();

        assert_eq!(res.value, Option::None);
    }

    #[tokio::test]
    async fn proper_vesting_emmisions() {
        let (vest, admin, recipient, asset) = setup().await;
        let cliff_timestamp = 5000;
        let end_timestamp = 10000;
        let total_amount = 10000;
        let cliff_amount = 3000;

        let vesting_schedule = [get_vesting_schedule(
            cliff_amount,
            cliff_timestamp,
            end_timestamp,
            0,
            total_amount,
            Identity::Address(recipient.address().into()),
            false,
        )];

        let _ = instantiate_vesting_contract(
            &vest,
            &admin.address().into(),
            &vesting_schedule.to_vec(),
            &asset,
            10000,
        )
        .await;

        let _ = mint_to_vesting(&asset, &vest, total_amount, &admin).await;

        let asset_id = AssetId::from(*asset.id().hash());

        let provider = admin.get_provider().unwrap();

        let vest_balance = provider
            .get_contract_asset_balance(&vest.id(), asset_id)
            .await
            .unwrap();

        assert_eq!(vest_balance, total_amount);
        // Time before cliff, no tokens should be redeemable
        let res = vest
            .methods()
            .get_redeemable_amount(
                cliff_timestamp - 1,
                Identity::Address(recipient.address().into()),
            )
            .call()
            .await
            .unwrap();
        assert_eq!(res.value, 0);

        // Time after end of vesting, all tokens should be redeemable
        let res = vest
            .methods()
            .get_redeemable_amount(
                end_timestamp + 1,
                Identity::Address(recipient.address().into()),
            )
            .call()
            .await
            .unwrap();

        assert_eq!(res.value, total_amount);

        // Midway through vesting, cliff + half of the remaining tokens should be redeemable
        let res = vest
            .methods()
            .get_redeemable_amount(
                cliff_timestamp + (end_timestamp - cliff_timestamp) / 2,
                Identity::Address(recipient.address().into()),
            )
            .call()
            .await
            .unwrap();

        assert_eq!(res.value, cliff_amount + (total_amount - cliff_amount) / 2);
    }
}
