use crate::utils::setup::setup;
use fuels::{prelude::ContractId, types::Identity};
mod success {
    use crate::utils::setup::test_helpers::{
        get_asset, get_vesting_schedule, instantiate_vesting_contract,
    };

    use super::*;

    #[tokio::test]
    async fn create_vesting_contract() {
        let (vest, admin, recipient) = setup().await;

        let vesting_schedule = [get_vesting_schedule(
            3000,
            1000,
            1000,
            0,
            10000,
            Identity::Address(recipient.address().into()),
            false,
        )];

        let asset = get_asset(ContractId::from([0u8; 32]), 10000);

        let _ = instantiate_vesting_contract(
            &vest,
            &Identity::Address(admin.address().into()),
            &vesting_schedule.to_vec(),
            &asset,
        )
        .await;

        let res = vest
            .methods()
            .get_vesting_schedule(Identity::Address(recipient.address().into()))
            .call()
            .await
            .unwrap();

        assert_eq!(res.value, vesting_schedule[0]);
        // println("res: {:?}", res.value);
    }

    #[tokio::test]
    async fn proper_vesting_emmisions() {
        let (vest, admin, recipient) = setup().await;
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

        let asset = get_asset(ContractId::from([0u8; 32]), 10000);

        let _ = instantiate_vesting_contract(
            &vest,
            &Identity::Address(admin.address().into()),
            &vesting_schedule.to_vec(),
            &asset,
        )
        .await;

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
