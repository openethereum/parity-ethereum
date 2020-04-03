#!/usr/bin/env bash

cargo build --release -p evmbin

./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmArithmeticTest
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBitwiseLogicOperation
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmBlockInfoTest
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmEnvironmentalInfo
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmIOandFlowOperations
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmLogTest
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPerformance
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmPushDupSwapTest
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmRandomTest
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSha3Test
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmSystemOperations
./target/release/openethereum-evm stats-jsontests-vm ./ethcore/res/ethereum/tests/VMTests/vmTests
