use super::test_common::*;
use super::chain::{ChainEra, json_chain_test};

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
