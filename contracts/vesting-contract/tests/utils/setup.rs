use fuels::prelude::*;

// TODO: do setup instead of copy/pasted code with minor adjustments

// Load abi from json
abigen!(
    Contract(
        name = "VestingContract",
        abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
    ),
    Contract(
        name = "MyAsset",
        abi = "contracts/vesting-contract/tests/artifacts/out/debug/asset-abi.json"
    )
);

const VESTING_CONTRACT_BINARY_PATH: &str = "out/debug/vesting-contract.bin";
const VESTING_CONTRACT_STORAGE_PATH: &str = "out/debug/vesting-contract-storage_slots.json";

pub const ASSET_CONTRACT_BINARY_PATH: &str = "./tests/artifacts/out/debug/asset.bin";
pub const ASSET_CONTRACT_STORAGE_PATH: &str =
    "./tests/artifacts/out/debug/asset-storage_slots.json";

pub async fn setup() -> (VestingContract, WalletUnlocked, WalletUnlocked, MyAsset) {
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;

    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let id = Contract::deploy(
        VESTING_CONTRACT_BINARY_PATH,
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(VESTING_CONTRACT_STORAGE_PATH.to_string())),
    )
    .await
    .unwrap();

    let instance = VestingContract::new(id.clone(), wallet.clone());

    let asset_id = Contract::deploy(
        ASSET_CONTRACT_BINARY_PATH,
        &wallet2,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(ASSET_CONTRACT_STORAGE_PATH.to_string())),
    )
    .await
    .unwrap();

    let asset = MyAsset::new(asset_id.clone(), wallet2.clone());

    (instance, wallet, wallet2, asset)
}

pub mod test_helpers {
    use fuels::programs::call_response::FuelCallResponse;
    use fuels::types::Identity;

    use super::abigen_bindings::vesting_contract_mod::{Asset, VestingSchedule};
    use super::*;

    pub async fn mint_to_vesting(
        contract: &MyAsset,
        recipient: &ContractId,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, fuels::types::errors::Error> {
        contract
            .methods()
            .mint_and_send_to_contract(amount, recipient.clone())
            .call()
            .await
    }

    pub async fn instantiate_vesting_contract(
        contract: &VestingContract,
        admin: &Identity,
        vesting_schedule: &Vec<VestingSchedule>,
        asset_contract: &MyAsset,
        amount: u64,
    ) -> Result<FuelCallResponse<()>, fuels::types::errors::Error> {
        let asset: Asset = Asset {
            id: asset_contract.id().into(),
            amount,
        };

        let _ = mint_to_vesting(asset_contract, &contract.id().into(), amount).await;

        contract
            .methods()
            .constructor(admin.clone(), vesting_schedule.clone(), asset.clone())
            .call()
            .await
    }

    pub fn get_vesting_schedule(
        cliff_amount: u64,
        cliff_timestamp: u64,
        end_timestamp: u64,
        claimed_amount: u64,
        total_amount: u64,
        recipient: Identity,
        revocable: bool,
    ) -> VestingSchedule {
        VestingSchedule {
            cliff_amount,
            cliff_timestamp,
            end_timestamp,
            claimed_amount,
            total_amount,
            recipient,
            revocable,
        }
    }
}
