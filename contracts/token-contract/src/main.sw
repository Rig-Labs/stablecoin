contract;
/*   
    ███████╗██╗    ██╗ █████╗ ██╗   ██╗     ██████╗  █████╗ ███╗   ██╗ ██████╗ 
    ██╔════╝██║    ██║██╔══██╗╚██╗ ██╔╝    ██╔════╝ ██╔══██╗████╗  ██║██╔════╝ 
    ███████╗██║ █╗ ██║███████║ ╚████╔╝     ██║  ███╗███████║██╔██╗ ██║██║  ███╗
    ╚════██║██║███╗██║██╔══██║  ╚██╔╝      ██║   ██║██╔══██║██║╚██╗██║██║   ██║
    ███████║╚███╔███╔╝██║  ██║   ██║       ╚██████╔╝██║  ██║██║ ╚████║╚██████╔╝
    ╚══════╝ ╚══╝╚══╝ ╚═╝  ╚═╝   ╚═╝        ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝                                                                         
*/

use libraries::token_interface::{Token, TokenInitializeConfig};

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


const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    config: TokenInitializeConfig = TokenInitializeConfig {
        name: "                                ",
        symbol: "        ",
        decimals: 1u8,
    },
    owner: Identity = Identity::Address(Address::from(ZERO_B256)),
    mint_amount: u64 = 0,
    mint_list: StorageMap<Identity, bool> = StorageMap {},
}

enum Error {
    AddressAlreadyMint: (),
    CannotReinitialize: (),
    MintIsClosed: (),
    NotOwner: (),
}


#[storage(read)]
fn validate_owner() {
    let sender = msg_sender().unwrap(); 
    require(storage.owner == sender, Error::NotOwner);
}

impl Token for Contract {
    //////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(config: TokenInitializeConfig, mint_amount: u64, owner: Identity) {
        require(storage.owner == Identity::Address(Address::from(ZERO_B256)), Error::CannotReinitialize);
        storage.owner = owner;
        storage.mint_amount = mint_amount;
        storage.config = config;
    }

    #[storage(read)]
    fn mint_to_id(amount: u64, address: Identity){
        validate_owner();
        mint_to(amount, address);
    }

    #[storage(read, write)]
    fn set_mint_amount(mint_amount: u64) {
        validate_owner();
        storage.mint_amount = mint_amount;
    }

    #[storage(read)]
    fn mint_coins(mint_amount: u64) {
        validate_owner();
        mint(mint_amount);
    }

    #[storage(read)]
    fn burn_coins(burn_amount: u64) {
        validate_owner();
        burn(burn_amount);
    }

    

    #[storage(read)]
    fn transfer_coins(coins: u64, address: Identity) {
        validate_owner();
        transfer(coins, contract_id(), address);
    }

    #[storage(read)]
    fn transfer_token_to_output(coins: u64, asset_id: ContractId, address: Identity) {
        validate_owner();
        transfer(coins, asset_id, address);
    }

    //////////////////////////////////////
    // Mint public method
    //////////////////////////////////////
    #[storage(read, write)]
    fn mint() {
        require(storage.mint_amount > 0, Error::MintIsClosed);

        // Enable a address to mint only once
        let sender = msg_sender().unwrap();
        require(storage.mint_list.get(sender) == false, Error::AddressAlreadyMint);

        storage.mint_list.insert(sender, true);
        mint_to(storage.mint_amount, sender);
    }

    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////
    #[storage(read)]
    fn get_mint_amount() -> u64 {
        storage.mint_amount
    }

    fn get_balance() -> u64 {
        balance_of(contract_id(), contract_id())
    }

    fn get_token_balance(asset_id: ContractId) -> u64 {
        balance_of(asset_id, contract_id())
    }
    #[storage(read)]
    fn config() -> TokenInitializeConfig {
        storage.config
    }

    #[storage(read)]
    fn already_minted(address: Identity) -> bool{
        storage.mint_list.get(address)
    }
}
