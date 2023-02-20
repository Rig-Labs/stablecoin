use crate::utils::setup::setup;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use fuels::prelude::*;
use fuels::{prelude::AssetId, types::Identity};

async fn produce_block_at_time(provider: &Provider, start_timestamp: u64) {
    let dt = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt((start_timestamp).try_into().unwrap(), 0).unwrap(),
        Utc,
    );

    let time: TimeParameters = TimeParameters {
        start_time: dt,
        block_time_interval: Duration::seconds(1),
    };

    let _res = provider.produce_blocks(1, Some(time)).await;
}


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

    #[tokio::test]
    async fn proper_claiming_vested_tokens() {
        let start_timestamp: u64 = Utc::now().timestamp().try_into().unwrap();
        // convert seconds to nano seconds;
        let (vest, admin, recipient, asset) = setup().await;
        let cliff_timestamp = (start_timestamp + 5000) * 1;
        let end_timestamp = (start_timestamp + 10000) * 1;
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

        produce_block_at_time(&provider, start_timestamp).await;

        let chain_info = provider.chain_info().await.unwrap();
        println!(
            "Current Simulated time: {:?}",
            chain_info.latest_block.header.time.unwrap()
        );

        println!(
            "Current Simulated time_stamp: {:?}",
            chain_info.latest_block.header.time.unwrap().timestamp()
        );
        // Time before cliff, no tokens should be redeemable


        let res = vest.methods().get_current_time().call().await.unwrap();

        println!(
            "Timestamp on smart contract: {} timestamp not propogating",
            res.value
        );
        println!("cliff_timestamp: {}", cliff_timestamp);

        let _res = vest
            .methods()
            .claim_vested_tokens(Identity::Address(recipient.address().into()))
            .append_variable_outputs(1)
            .call()
            .await;

        let rec_balance = provider
            .get_asset_balance(&recipient.address(), asset_id)
            .await
            .unwrap();

        println!("Reciever balance after claiming: {}", rec_balance);
        
        println!("Minting asset...to admin: {:?}", admin.address());

        let _res = asset
            .methods()
            .mint_and_send_to_address(10000, admin.address().into())
            .append_variable_outputs(1)
            .call()
            .await;
        // println!("Minting result: {:?}", res);

        let asset_id = AssetId::from(*asset.id().hash());

        let provider = admin.get_provider().unwrap();

        let balance = provider
            .get_asset_balance(&admin.address(), asset_id)
            .await
            .unwrap();

        println!("Admin balance: {:?}", balance);

        let _ = admin
            .force_transfer_to_contract(&vest.id(), 99, asset_id, TxParameters::default())
            .await;

        let balance = provider
            .get_asset_balance(&admin.address(), asset_id)
            .await
            .unwrap();

        println!("Admin balance: {:?}", balance);

        let vest_balance = provider
            .get_contract_asset_balance(&vest.id(), asset_id)
            .await
            .unwrap();

        println!("Vest balance: {:?}", vest_balance);
        // TODO Check balances
    }
}
