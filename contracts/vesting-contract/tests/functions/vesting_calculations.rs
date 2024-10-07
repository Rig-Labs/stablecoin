use crate::utils::setup::setup;
use fuels::prelude::*;
use fuels::types::transaction_builders::VariableOutputPolicy;
use fuels::types::Identity;

mod success {

    use test_utils::interfaces::vesting::{
        get_vesting_schedule, instantiate_vesting_contract, load_vesting_schedules_from_json_file,
        set_timestamp, VestingContract,
    };

    use crate::utils::setup::test_helpers::init_and_mint_to_vesting;

    use super::*;

    #[tokio::test]
    async fn create_vesting_contract() {
        let (vest, admin, recipient, asset) = setup(10000).await;

        let vesting_schedule = [get_vesting_schedule(
            3000,
            1000,
            2000,
            0,
            10000,
            Identity::Address(recipient.address().into()),
        )];

        let _ = instantiate_vesting_contract(
            &vest,
            &asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            vesting_schedule.to_vec(),
        )
        .await;

        let res = vest
            .methods()
            .get_vesting_schedule(Identity::Address(recipient.address().into()))
            .call()
            .await
            .unwrap();

        assert_eq!(res.value, vesting_schedule[0]);

        vest.methods()
            .get_vesting_schedule(Identity::Address(admin.address().into()))
            .call()
            .await
            .unwrap_err();

        // fails to initialize twice
        let res = instantiate_vesting_contract(
            &vest,
            &asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            vesting_schedule.to_vec(),
        )
        .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn proper_vesting_calculations() {
        let (vest, admin, recipient, asset) = setup(10000).await;
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
        )];

        let _ = instantiate_vesting_contract(
            &vest,
            &asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            vesting_schedule.to_vec(),
        )
        .await;

        let _ = init_and_mint_to_vesting(&asset, &vest, total_amount, &admin).await;

        let asset_id = asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let provider = admin.provider().unwrap();

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

    #[tokio::test]
    async fn proper_claiming_vested_tokens() {
        let start_timestamp: u64 = 0;
        // convert seconds to nano seconds;
        let (vest, admin, recipient, asset) = setup(10000).await;
        let cliff_timestamp = (start_timestamp + 100) * 1;
        let end_timestamp = (cliff_timestamp + 100) * 1;
        let total_amount = 10000;
        let cliff_amount = 3000;

        let recpient_vesting = VestingContract::new(vest.contract_id().clone(), recipient.clone());

        let vesting_schedule = [get_vesting_schedule(
            cliff_amount,
            cliff_timestamp,
            end_timestamp,
            0,
            total_amount,
            Identity::Address(recipient.address().into()),
        )];

        let _ = instantiate_vesting_contract(
            &vest,
            &asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            vesting_schedule.to_vec(),
        )
        .await;

        let _ = init_and_mint_to_vesting(&asset, &vest, total_amount, &admin).await;

        let asset_id = asset
            .contract_id()
            .asset_id(&AssetId::zeroed().into())
            .into();

        let provider = admin.provider().unwrap();

        let starting_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(starting_balance, 0);

        let vest_balance = provider
            .get_contract_asset_balance(&vest.id(), asset_id)
            .await
            .unwrap();

        assert_eq!(vest_balance, total_amount);

        let _ = set_timestamp(&vest, 20).await;

        let _res = recpient_vesting
            .methods()
            .claim_vested_tokens()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(rec_balance, 0);

        let _ = set_timestamp(&vest, cliff_timestamp).await;
        // Block produced then claim vested tokens happens in the next block

        let _res = recpient_vesting
            .methods()
            .claim_vested_tokens()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(cliff_amount, rec_balance);

        let _ = set_timestamp(&vest, end_timestamp - (end_timestamp - cliff_timestamp) / 2).await;

        // Block produced then claim vested tokens happens in the next block
        let _res = recpient_vesting
            .methods()
            .claim_vested_tokens()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(
            cliff_amount + (total_amount - cliff_amount) / 2,
            rec_balance
        );

        let _ = set_timestamp(&vest, end_timestamp).await;
        // Block produced then claim vested tokens happens in the next block

        let _res = recpient_vesting
            .methods()
            .claim_vested_tokens()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(total_amount, rec_balance);

        set_timestamp(&vest, end_timestamp + 10).await;

        // Tries to claim after all tokens have been claimed
        let _res = recpient_vesting
            .methods()
            .claim_vested_tokens()
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        assert_eq!(total_amount, rec_balance);
    }

    #[tokio::test]
    async fn proper_json_vesting_parsing() {
        let vesting_schedules = load_vesting_schedules_from_json_file(
            "contracts/vesting-contract/tests/artefacts/test_vesting.json",
        );

        assert_eq!(vesting_schedules.len(), 2);
        assert_eq!(vesting_schedules[0].cliff_amount, 1);
        assert_eq!(vesting_schedules[0].cliff_timestamp, 2);
        assert_eq!(vesting_schedules[0].end_timestamp, 3);
        assert_eq!(vesting_schedules[0].claimed_amount, 4);
        assert_eq!(vesting_schedules[0].total_amount, 5);

        assert_eq!(vesting_schedules[1].cliff_amount, 6);
        assert_eq!(vesting_schedules[1].cliff_timestamp, 7);
        assert_eq!(vesting_schedules[1].end_timestamp, 8);
        assert_eq!(vesting_schedules[1].claimed_amount, 9);
        assert_eq!(vesting_schedules[1].total_amount, 10);
    }
}

mod failure {
    use test_utils::interfaces::vesting::{
        get_vesting_schedule, instantiate_vesting_contract, VestingContract,
    };

    use super::*;

    #[tokio::test]
    async fn fails_to_initialize_vesting_with_incorrect_total_amount() {
        let total_amount = 10000;
        let (vest, _, recipient, asset) = setup(total_amount).await;

        let vesting_schedule = [get_vesting_schedule(
            3000,
            1000,
            2000,
            0,
            total_amount + 1,
            Identity::Address(recipient.address().into()),
        )];

        let res = instantiate_vesting_contract(
            &vest,
            &asset
                .contract_id()
                .asset_id(&AssetId::zeroed().into())
                .into(),
            vesting_schedule.to_vec(),
        )
        .await;

        assert!(res.is_err());
    }
}
