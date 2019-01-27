#!/usr/bin/env bash

cargo build --release -p evmbin

./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmArithmeticTest
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBitwiseLogicOperation
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBlockInfoTest
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmEnvironmentalInfo
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmIOandFlowOperations
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmLogTest
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPerformance
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPushDupSwapTest
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmRandomTest
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSha3Test
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSystemOperations
./target/release/parity-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmTests
