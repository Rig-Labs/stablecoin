use fuels::{
    prelude::{abigen, AssetId, CallParameters, ContractId, TxParameters},
    programs::call_response::FuelCallResponse,
};

abigen!(
    Contract(
        name = "VestingContract",
        abi = "contracts/vesting-contract/out/debug/vesting-contract-abi.json"
    ),
    Contract(
        name = "Token",
        abi = "contracts/token-contract/out/debug/token-contract-abi.json"
    )
);

pub mod token {
    use super::*;
}
