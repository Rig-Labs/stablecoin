# Fluid Protocol

## [`Good Sway Contract Reference`](https://github.com/FuelLabs/sway-applications/tree/master/AMM/project)

Contracts
---------

The source code for each contract is in the [`contracts/`](contracts/)
directory.

| Name                                               | Description                            | Status |
| -------------------------------------------------- | -------------------------------------- | ------- |
| [`mock-oracle`](contracts/mock-oracle-contract)       | Oracle for on-chain data | $$\color{green}{Completed}$$ 
| [`vesting`](contracts/vesting-contract)       | Manages $FPT vesting schedules | $$\color{orange}{In}$$ $$\color{orange}{Progress}$$ 
| [`protocol-factory`](contracts/protocol-contract)       | Routes risk functions to riskies trove from all trove managers, instatiates everything |$$\color{red}{Not}$$ $$\color{red}{Started}$$ 
| [`trove-manager`](contracts/trove-manager-contract)       | Manages minting $USDF, liquidations, and user troves in the Linked list format |$$\color{red}{Not}$$ $$\color{red}{Started}$$ 
| [`stability-pool`](contracts/stability-pool-contract)       | Manages desposits to liquidate user troves | $$\color{red}{Not}$$ $$\color{red}{Started}$$ |
| [`staking`](contracts/staking-contract)       | Manages $FPT staking emissions from fee collection | $$\color{red}{Not}$$ $$\color{red}{Started}$$ |

Build + Test Contracts
-------------------------------

Make sure you have fuelup, fuel-core, cargo, and rust installed 

```
sh build-and-test.sh
```

License
-------

MIT License (see `/LICENSE`)
