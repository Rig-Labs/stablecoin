# Fluid Protocol

## [`Good Sway Contract Reference`](https://github.com/FuelLabs/sway-applications/tree/master/AMM/project)

Contracts
---------

The source code for each contract is in the [`contracts/`](contracts/)
directory.

| Name                                               | Description                            | Status |
| -------------------------------------------------- | -------------------------------------- | ------- |
| [`mock-oracle`](contracts/mock-oracle-contract)       | Oracle for on-chain data | $$\color{green}{95/100}$$ 
| [`token`](contracts/token-contract)       | FRC-20 to use in local tests made by sway gang | $$\color{green}{95/100}$$ 
| [`sorted-troves`](contracts/sorted-troves-contract)       | Manages data of troves in the Linked list format |$$\color{green}{90/100}$$
| [`vesting`](contracts/vesting-contract)       | Manages $FPT vesting schedules | $$\color{orange}{85/100}$$
| [`borrow-operations`](contracts/borrow-operations-contract)       | Interface with which users manager their troves | $$\color{orange}{40/100}$$ |
| [`trove-manager`](contracts/trove-manager-contract)       | Manages minting $USDF, liquidations, and user troves in the Linked list format |$$\color{orange}{10/100}$$
| [`stability-pool`](contracts/stability-pool-contract)       | Manages desposits to liquidate user troves | $$\color{red}{0/100}$$
| [`staking`](contracts/staking-contract)       | Manages $FPT staking emissions from fee collection | $$\color{red}{0/100}$$ |
| [`protocol-factory`](contracts/protocol-contract)       | Routes risk functions to riskies trove from all trove managers, instatiates everything |$$\color{red}{0/100}$$

Build + Test Contracts
-------------------------------

Make sure you have fuelup, fuel-core, cargo, and rust installed 

```
sh build-and-test.sh
```

Functionality
-------------------------------
- [x] Create Trove and Recieve $USDF
- [x] Add more collateral to trove
- [ ] Remove more collateral to trove
- [ ] Repay Loan
- [ ] Close Trove
- [ ] Liquidate
- [ ] Stake FPT
- [ ] Stability Pool

License
-------

MIT License (see `/LICENSE`)
