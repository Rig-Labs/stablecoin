contract;

use libraries::community_issuance_interface::{CommunityIssuance};

use std::{
    address::*,
    auth::{
        AuthError,
        msg_sender,
    },
    identity::{Identity},
    call_frames::{contract_id, msg_asset_id},
    context::{balance_of, msg_amount},
    contract_id::ContractId,
    revert::require,
    storage::*,
    token::*,
};

storage {

}

impl CommunityIssuance for Contract {
    #[storage(read, write)]
    fn initialize() {

    }
}
