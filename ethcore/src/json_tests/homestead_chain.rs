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
use super::chain::json_chain_test;
use tests::helpers::*;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	json_chain_test(json_data, ChainEra::Homestead)
}

declare_test!{BlockchainTests_Homestead_bcBlockGasLimitTest, "BlockchainTests/Homestead/bcBlockGasLimitTest"}
declare_test!{BlockchainTests_Homestead_bcForkStressTest, "BlockchainTests/Homestead/bcForkStressTest"}
declare_test!{BlockchainTests_Homestead_bcGasPricerTest, "BlockchainTests/Homestead/bcGasPricerTest"}
declare_test!{BlockchainTests_Homestead_bcInvalidHeaderTest, "BlockchainTests/Homestead/bcInvalidHeaderTest"}
declare_test!{BlockchainTests_Homestead_bcInvalidRLPTest, "BlockchainTests/Homestead/bcInvalidRLPTest"}
declare_test!{BlockchainTests_Homestead_bcMultiChainTest, "BlockchainTests/Homestead/bcMultiChainTest"}
declare_test!{BlockchainTests_Homestead_bcRPC_API_Test, "BlockchainTests/Homestead/bcRPC_API_Test"}
declare_test!{BlockchainTests_Homestead_bcStateTest, "BlockchainTests/Homestead/bcStateTest"}
declare_test!{BlockchainTests_Homestead_bcTotalDifficultyTest, "BlockchainTests/Homestead/bcTotalDifficultyTest"}
declare_test!{BlockchainTests_Homestead_bcUncleHeaderValiditiy, "BlockchainTests/Homestead/bcUncleHeaderValiditiy"}
declare_test!{BlockchainTests_Homestead_bcUncleTest, "BlockchainTests/Homestead/bcUncleTest"}
declare_test!{BlockchainTests_Homestead_bcValidBlockTest, "BlockchainTests/Homestead/bcValidBlockTest"}
declare_test!{BlockchainTests_Homestead_bcWalletTest, "BlockchainTests/Homestead/bcWalletTest"}
declare_test!{BlockchainTests_Homestead_bcShanghaiLove, "BlockchainTests/Homestead/bcShanghaiLove"}
declare_test!{BlockchainTests_Homestead_bcSuicideIssue, "BlockchainTests/Homestead/bcSuicideIssue"}
declare_test!{BlockchainTests_Homestead_bcExploitTest, "BlockchainTests/Homestead/bcExploitTest"}
