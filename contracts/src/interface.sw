library interface;

abi FluidProtocol {
    fn test_function() -> bool;
}

abi PriceFeed {
    fn get_fuel_price() -> u256;
}
