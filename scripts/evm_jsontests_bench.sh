#!/usr/bin/env bash

cargo build --release -p evmbin

./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmArithmeticTest --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBitwiseLogicOperation --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBlockInfoTest --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmEnvironmentalInfo --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmIOandFlowOperations --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmLogTest --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPerformance --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPushDupSwapTest --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmRandomTest --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSha3Test --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSystemOperations --folder
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmTests --folder
