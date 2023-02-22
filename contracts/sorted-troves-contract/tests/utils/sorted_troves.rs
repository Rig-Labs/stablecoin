use fuels::prelude::*;
use fuels::programs::call_response::FuelCallResponse;
use fuels::types::Identity;

abigen!(
    Contract(
        name = "SortedTroves",
        abi = "contracts/sorted-troves-contract/out/debug/sorted-troves-contract-abi.json"
    ),
    Contract(
        name = "TroveManagerContract",
        abi = "contracts/trove-manager-contract/out/debug/trove-manager-contract-abi.json"
    )
);

pub mod sorted_troves_abi_calls {

    use super::*;
}
