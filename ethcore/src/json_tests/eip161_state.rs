// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use super::test_common::*;
use tests::helpers::*;
use super::state::json_chain_test;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	json_chain_test(json_data, ChainEra::Eip161)
}

declare_test!{StateTests_EIP158_stEIP158SpecificTest, "StateTests/EIP158/stEIP158SpecificTest"}
declare_test!{StateTests_EIP158_stNonZeroCallsTest, "StateTests/EIP158/stNonZeroCallsTest"}
declare_test!{StateTests_EIP158_stZeroCallsTest, "StateTests/EIP158/stZeroCallsTest"}

declare_test!{StateTests_EIP158_EIP150_stMemExpandingEIPCalls, "StateTests/EIP158/EIP150/stMemExpandingEIPCalls"}
declare_test!{StateTests_EIP158_EIP150_stEIPSpecificTest, "StateTests/EIP158/EIP150/stEIPSpecificTest"}
declare_test!{StateTests_EIP158_EIP150_stEIPsingleCodeGasPrices, "StateTests/EIP158/EIP150/stEIPsingleCodeGasPrices"}
declare_test!{StateTests_EIP158_EIP150_stChangedTests, "StateTests/EIP158/EIP150/stChangedTests"}

declare_test!{StateTests_EIP158_Homestead_stBoundsTest, "StateTests/EIP158/Homestead/stBoundsTest"}
declare_test!{StateTests_EIP158_Homestead_stCallCodes, "StateTests/EIP158/Homestead/stCallCodes"}
declare_test!{StateTests_EIP158_Homestead_stCallCreateCallCodeTest, "StateTests/EIP158/Homestead/stCallCreateCallCodeTest"}
declare_test!{StateTests_EIP158_Homestead_stCallDelegateCodes, "StateTests/EIP158/Homestead/stCallDelegateCodes"}
declare_test!{StateTests_EIP158_Homestead_stCallDelegateCodesCallCode, "StateTests/EIP158/Homestead/stCallDelegateCodesCallCode"}
declare_test!{StateTests_EIP158_Homestead_stDelegatecallTest, "StateTests/EIP158/Homestead/stDelegatecallTest"}
declare_test!{StateTests_EIP158_Homestead_stHomeSteadSpecific, "StateTests/EIP158/Homestead/stHomeSteadSpecific"}
declare_test!{StateTests_EIP158_Homestead_stInitCodeTest, "StateTests/EIP158/Homestead/stInitCodeTest"}
declare_test!{StateTests_EIP158_Homestead_stLogTests, "StateTests/EIP158/Homestead/stLogTests"}
declare_test!{heavy => StateTests_EIP158_Homestead_stMemoryTest, "StateTests/EIP158/Homestead/stMemoryTest"}
declare_test!{StateTests_EIP158_Homestead_stPreCompiledContracts, "StateTests/EIP158/Homestead/stPreCompiledContracts"}
declare_test!{heavy => StateTests_EIP158_Homestead_stQuadraticComplexityTest, "StateTests/EIP158/Homestead/stQuadraticComplexityTest"}
declare_test!{StateTests_EIP158_Homestead_stRecursiveCreate, "StateTests/EIP158/Homestead/stRecursiveCreate"}
declare_test!{StateTests_EIP158_Homestead_stRefundTest, "StateTests/EIP158/Homestead/stRefundTest"}
declare_test!{StateTests_EIP158_Homestead_stSpecialTest, "StateTests/EIP158/Homestead/stSpecialTest"}
declare_test!{StateTests_EIP158_Homestead_stSystemOperationsTest, "StateTests/EIP158/Homestead/stSystemOperationsTest"}
declare_test!{StateTests_EIP158_Homestead_stTransactionTest, "StateTests/EIP158/Homestead/stTransactionTest"}
declare_test!{StateTests_EIP158_Homestead_stWalletTest, "StateTests/EIP158/Homestead/stWalletTest"}