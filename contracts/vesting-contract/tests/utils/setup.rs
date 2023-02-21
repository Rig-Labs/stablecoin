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
        vesting_contract: &VestingContract,
        amount: u64,
        admin: &WalletUnlocked,
    ) {
        let asset_id = AssetId::from(*contract.id().hash());

        let _ = contract
            .methods()
            .mint_and_send_to_address(amount, admin.address().into())
            .append_variable_outputs(1)
            .call()
            .await;

        let _ = admin
            .force_transfer_to_contract(
                &vesting_contract.id(),
                amount,
                asset_id,
                TxParameters::default(),
            )
            .await;
    }

    pub async fn instantiate_vesting_contract(
        contract: &VestingContract,
        admin: &Address,
        vesting_schedule: &Vec<VestingSchedule>,
        asset_contract: &MyAsset,
        amount: u64,
    ) -> Result<FuelCallResponse<()>> {
        let asset: Asset = Asset {
            id: asset_contract.id().into(),
            amount,
        };

        contract
            .methods()
            .constructor(
                Identity::Address(admin.clone()),
                vesting_schedule.clone(),
                asset.clone(),
            )
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
