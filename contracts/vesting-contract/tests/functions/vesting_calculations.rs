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
            200,
            1000,
            9000,
            0,
            0,
            1000,
            Identity::Address(recipient.address().into()),
            false,
        )];

        let asset = get_asset(ContractId::from([0u8; 32]), 1000);

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

        println!("{:?}", res);
    }
}
