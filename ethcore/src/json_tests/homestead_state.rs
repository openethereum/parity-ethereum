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
	json_chain_test(json_data, ChainEra::Homestead)
}

declare_test!{StateTests_Homestead_stCallCodes, "StateTests/Homestead/stCallCodes"}
declare_test!{StateTests_Homestead_stCallCreateCallCodeTest, "StateTests/Homestead/stCallCreateCallCodeTest"}
declare_test!{StateTests_Homestead_stDelegatecallTest, "StateTests/Homestead/stDelegatecallTest"}
declare_test!{StateTests_Homestead_stInitCodeTest, "StateTests/Homestead/stInitCodeTest"}
declare_test!{StateTests_Homestead_stLogTests, "StateTests/Homestead/stLogTests"}
declare_test!{heavy => StateTests_Homestead_stMemoryStressTest, "StateTests/Homestead/stMemoryStressTest"}
declare_test!{heavy => StateTests_Homestead_stMemoryTest, "StateTests/Homestead/stMemoryTest"}
declare_test!{StateTests_Homestead_stPreCompiledContracts, "StateTests/Homestead/stPreCompiledContracts"}
declare_test!{heavy => StateTests_Homestead_stQuadraticComplexityTest, "StateTests/Homestead/stQuadraticComplexityTest"}
declare_test!{StateTests_Homestead_stRecursiveCreate, "StateTests/Homestead/stRecursiveCreate"}
declare_test!{StateTests_Homestead_stRefundTest, "StateTests/Homestead/stRefundTest"}
declare_test!{StateTests_Homestead_stSpecialTest, "StateTests/Homestead/stSpecialTest"}
declare_test!{StateTests_Homestead_stSystemOperationsTest, "StateTests/Homestead/stSystemOperationsTest"}
declare_test!{StateTests_Homestead_stTransactionTest, "StateTests/Homestead/stTransactionTest"}
declare_test!{StateTests_Homestead_stWalletTest, "StateTests/Homestead/stWalletTest"}
