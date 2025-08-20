// References:
// https://docs.pyth.network/price-feeds/contract-addresses/fuel
// https://github.com/FuelLabs/verified-assets/blob/main/ASSETS.md

pub const TESTNET_CONTRACTS_FILE: &str = "testnet.contracts.json";
pub const MAINNET_CONTRACTS_FILE: &str = "mainnet.contracts.json";

pub const TESTNET_RPC: &str = "https://testnet.fuel.network/v1/playground";
pub const MAINNET_RPC: &str = "https://mainnet.fuel.network/v1/playground";

pub const TESTNET_PYTH_CONTRACT_ID: &str =
    "0x25146735b29d4216639f7f8b1d7b921ff87a1d3051de62d6cceaacabeb33b8e7";
pub const MAINNET_PYTH_CONTRACT_ID: &str =
    "0x1c86fdd9e0e7bc0d2ae1bf6817ef4834ffa7247655701ee1b031b52a24c523da";

pub const TESTNET_STORK_CONTRACT_ID: &str =
    "0x09c88f50d535ac5ce8945e34c418233b1e3834be9a88effb57cb137321fbae0c";
pub const MAINNET_STORK_CONTRACT_ID: &str =
    "0x9c118ae13927dd51ba59c0370dc8c272a3b64ccd675950750c8840a649c81149";

// Stork Price Feeds
pub const STORK_ETH_PRICE_ID: &str =
    "0x59102b37de83bdda9f38ac8254e596f0d9ac61d2035c07936675e87342817160";
pub const STORK_FUEL_PRICE_ID: &str =
    "0x670b7091d54af59331f97a1ce4a321eab14fd257a8b57b75ce4d4a5afc9186f4";
pub const STORK_STFUEL_PRICE_ID: &str =
    "0xa3ed2e58076f53e8dd15c8463ee49e6ce547355c34c639777c5dace3728e2ded";

// Testnet
pub const TESTNET_ETH_ASSET_CONTRACT_ID: &str =
    "0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8";
pub const TESTNET_ETH_ASSET_ID: &str =
    "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07";

// Mainnet
// https://github.com/FuelLabs/verified-assets/blob/main/ASSETS.md
pub const MAINNET_ASSET_CONTRACT_ID: &str =
    "0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8";
pub const MAINNET_ETH_ASSET_ID: &str =
    "0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07";
pub const MAINNET_ETH_DECIMALS: u32 = 9;
pub const MAINNET_WSTETH_ASSET_ID: &str =
    "0x1a7815cc9f75db5c24a5b0814bfb706bb9fe485333e98254015de8f48f84c67b";
pub const MAINNET_WSTETH_DECIMALS: u32 = 9;
pub const MAINNET_EZETH_ASSET_ID: &str =
    "0x91b3559edb2619cde8ffb2aa7b3c3be97efd794ea46700db7092abeee62281b0";
pub const MAINNET_EZETH_DECIMALS: u32 = 9;
pub const MAINNET_PZETH_ASSET_ID: &str =
    "0x1493d4ec82124de8f9b625682de69dcccda79e882b89a55a8c737b12de67bd68";
pub const MAINNET_PZETH_DECIMALS: u32 = 9;
pub const MAINNET_WEETH_ASSET_ID: &str =
    "0x239ed6e12b7ce4089ee245244e3bf906999a6429c2a9a445a1e1faf56914a4ab";
pub const MAINNET_WEETH_DECIMALS: u32 = 9;
pub const MAINNET_RSETH_ASSET_ID: &str =
    "0xbae80f7fb8aa6b90d9b01ef726ec847cc4f59419c4d5f2ea88fec785d1b0e849";
pub const MAINNET_RSETH_DECIMALS: u32 = 9;
pub const MAINNET_METH_ASSET_ID: &str =
    "0xafd219f513317b1750783c6581f55530d6cf189a5863fd18bd1b3ffcec1714b4";
pub const MAINNET_METH_DECIMALS: u32 = 9;
pub const MAINNET_FUEL_ASSET_ID: &str =
    "0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82";
pub const MAINNET_FUEL_DECIMALS: u32 = 9;
pub const MAINNET_STFUEL_DECIMALS: u32 = 9;

// pyth price ids
// https://www.pyth.network/developers/price-feed-ids#pyth-evm-stable
pub const PYTH_ETH_PRICE_ID: &str =
    "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace";
pub const PYTH_WSTETH_PRICE_ID: &str =
    "6df640f3b8963d8f8358f791f352b8364513f6ab1cca5ed3f1f7b5448980e784";
pub const PYTH_EZETH_PRICE_ID: &str =
    "06c217a791f5c4f988b36629af4cb88fad827b2485400a358f3b02886b54de92";
pub const PYTH_WEETH_PRICE_ID: &str =
    "9ee4e7c60b940440a261eb54b6d8149c23b580ed7da3139f7f08f4ea29dad395";
pub const PYTH_RSETH_PRICE_ID: &str =
    "0caec284d34d836ca325cf7b3256c078c597bc052fbd3c0283d52b581d68d71f";
pub const PYTH_METH_PRICE_ID: &str =
    "fbc9c3a716650b6e24ab22ab85b1c0ef4141b18f4590cc0b986e2f9064cf73d6";
pub const PYTH_FUEL_PRICE_ID: &str =
    "8a757d54e5d34c7ff1aea8502a2d968686027a304d00418092aaf7e60ed98d95";
pub const PYTH_PZETH_PRICE_ID: &str = ""; // Waiting for Pyth price feed

pub struct AssetConstants {
    pub symbol: &'static str,
    pub asset_contract_id: Option<&'static str>,
    pub asset_id: Option<&'static str>,
    pub pyth_contract_id: Option<&'static str>,
    pub pyth_price_id: Option<&'static str>,
    pub stork_contract_id: Option<&'static str>,
    pub stork_price_id: Option<&'static str>,
    pub decimals: u32,
}

// Asset-specific constants

pub const TESTNET_FUEL_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "FUEL",
    asset_contract_id: Some("0x922218A7595D4Fd6F489280040Dc768754A40b5c027b3f2710698B5A38B1066D"),
    asset_id: Some("0xe71e90bad4aa7d15edcd951c473f33f566c048ab304f2e3367fc7555efb5dc37"),
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_FUEL_PRICE_ID),
    decimals: 9,
    stork_contract_id: Some(TESTNET_STORK_CONTRACT_ID),
    stork_price_id: Some(STORK_FUEL_PRICE_ID),
};

pub const TESTNET_STFUEL_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "STFUEL",
    asset_contract_id: Some("0xf25234BB775948c0fD3C685F139D6C6B5724d060968bc729aA3F527f1d7DBA10"),
    asset_id: Some("0xfd6beecb8e6229aaf981727ba56af0296e9b3ba4efef75473398fdf9a952a2a8"),
    pyth_contract_id: None,
    pyth_price_id: None,
    decimals: 9,
    stork_contract_id: Some(TESTNET_STORK_CONTRACT_ID),
    stork_price_id: Some(STORK_STFUEL_PRICE_ID),
};

pub const TESTNET_ETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "ETH",
    asset_contract_id: Some("0x7C4c0454E683ca3b0aa6191f00Ce635895Dbcad391e77cF8520A54fDE47A017d"),
    asset_id: Some("0x90cfbc7e7001f42344f0bfd9f78a11da99eebf6cabd4d1d136f6a8526840f02e"),
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_ETH_PRICE_ID),
    decimals: 9,
    stork_contract_id: Some(TESTNET_STORK_CONTRACT_ID),
    stork_price_id: Some(STORK_ETH_PRICE_ID),
};

pub const TESTNET_WSTETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "WSTETH",
    asset_contract_id: None,
    asset_id: None,
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_WSTETH_PRICE_ID),
    decimals: 9,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const TESTNET_EZETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "EZETH",
    asset_contract_id: None,
    asset_id: None,
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_EZETH_PRICE_ID),
    decimals: 9,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const TESTNET_WEETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "WEETH",
    asset_contract_id: None,
    asset_id: None,
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_WEETH_PRICE_ID),
    decimals: 9,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const TESTNET_RSETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "RSETH",
    asset_contract_id: None,
    asset_id: None,
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_RSETH_PRICE_ID),
    decimals: 9,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const TESTNET_METH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "METH",
    asset_contract_id: None,
    asset_id: None,
    pyth_contract_id: Some(TESTNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_METH_PRICE_ID),
    decimals: 9,
    stork_contract_id: None,
    stork_price_id: None,
};

// Mainnet

pub const MAINNET_FUEL_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "FUEL",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_FUEL_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_FUEL_PRICE_ID),
    decimals: MAINNET_FUEL_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_ETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "ETH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_ETH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_ETH_PRICE_ID),
    decimals: MAINNET_ETH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_WSTETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "WSTETH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_WSTETH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_WSTETH_PRICE_ID),
    decimals: MAINNET_WSTETH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_EZETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "EZETH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_EZETH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_EZETH_PRICE_ID),
    decimals: MAINNET_EZETH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_WEETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "WEETH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_WEETH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_WEETH_PRICE_ID),
    decimals: MAINNET_WEETH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_RSETH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "RSETH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_RSETH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_RSETH_PRICE_ID),
    decimals: MAINNET_RSETH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

pub const MAINNET_METH_CONSTANTS: AssetConstants = AssetConstants {
    symbol: "METH",
    asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
    asset_id: Some(MAINNET_METH_ASSET_ID),
    pyth_contract_id: Some(MAINNET_PYTH_CONTRACT_ID),
    pyth_price_id: Some(PYTH_METH_PRICE_ID),
    decimals: MAINNET_METH_DECIMALS,
    stork_contract_id: None,
    stork_price_id: None,
};

// pub const MAINNET_PZETH_CONSTANTS: AssetConstants = AssetConstants {
//     symbol: "PZETH",
//     asset_contract_id: Some(MAINNET_ASSET_CONTRACT_ID),
//     asset_id: Some(MAINNET_PZETH_ASSET_ID),
//     pyth_contract_id: MAINNET_PYTH_CONTRACT_ID,
//     pyth_price_id: PYTH_PZETH_PRICE_ID,
//     decimals: MAINNET_PZETH_DECIMALS,
// };
