// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! rpc integration tests.
use std::env;
use std::sync::Arc;

use accounts::AccountProvider;
use ethcore::client::{BlockChainClient, Client, ClientConfig, ChainInfo, ImportBlock};
use ethcore::ethereum;
use ethcore::miner::Miner;
use ethcore::spec::{Genesis, Spec};
use ethcore::test_helpers;
use ethcore::verification::VerifierType;
use ethcore::verification::queue::kind::blocks::Unverified;
use ethereum_types::{Address, H256, U256};
use ethjson::blockchain::BlockChain;
use ethjson::spec::ForkSpec;
use io::IoChannel;
use miner::external::ExternalMiner;
use parity_runtime::Runtime;
use parking_lot::Mutex;
use types::ids::BlockId;

use jsonrpc_core::IoHandler;
use v1::helpers::dispatch::{self, FullDispatcher};
use v1::helpers::nonce;
use v1::impls::{EthClient, EthClientOptions, SigningUnsafeClient};
use v1::metadata::Metadata;
use v1::tests::helpers::{TestSnapshotService, TestSyncProvider, Config};
use v1::traits::{Eth, EthSigning};

fn account_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn sync_provider() -> Arc<TestSyncProvider> {
	Arc::new(TestSyncProvider::new(Config {
		network_id: 3,
		num_peers: 120,
	}))
}

fn miner_service(spec: &Spec) -> Arc<Miner> {
	Arc::new(Miner::new_for_tests(spec, None))
}

fn snapshot_service() -> Arc<TestSnapshotService> {
	Arc::new(TestSnapshotService::new())
}

fn make_spec(chain: &BlockChain) -> Spec {
	let genesis = Genesis::from(chain.genesis());
	let mut spec = ethereum::new_frontier_test();
	let state = chain.pre_state.clone().into();
	spec.set_genesis_state(state).expect("unable to set genesis state");
	spec.overwrite_genesis_params(genesis);
	assert!(spec.is_state_root_valid());
	spec
}

struct EthTester {
	_miner: Arc<Miner>,
	_runtime: Runtime,
	_snapshot: Arc<TestSnapshotService>,
	accounts: Arc<AccountProvider>,
	client: Arc<Client>,
	handler: IoHandler<Metadata>,
}

impl EthTester {
	fn from_chain(chain: &BlockChain) -> Self {

		let tester = if ::ethjson::blockchain::Engine::NoProof == chain.engine {
			let mut config = ClientConfig::default();
			config.verifier_type = VerifierType::CanonNoSeal;
			config.check_seal = false;
			Self::from_spec_conf(make_spec(chain), config)
		} else {
			Self::from_spec(make_spec(chain))
		};

		for b in chain.blocks_rlp() {
			if let Ok(block) = Unverified::from_rlp(b) {
				let _ = tester.client.import_block(block);
				tester.client.flush_queue();
				tester.client.import_verified_blocks();
			}
		}

		tester.client.flush_queue();

		assert!(tester.client.chain_info().best_block_hash == chain.best_block.clone().into());
		tester
	}

	fn from_spec(spec: Spec) -> Self {
		let config = ClientConfig::default();
		Self::from_spec_conf(spec, config)
	}

	fn from_spec_conf(spec: Spec, config: ClientConfig) -> Self {
		let runtime = Runtime::with_thread_count(1);
		let account_provider = account_provider();
		let ap = account_provider.clone();
		let accounts  = Arc::new(move || ap.accounts().unwrap_or_default()) as _;
		let miner_service = miner_service(&spec);
		let snapshot_service = snapshot_service();

		let client = Client::new(
			config,
			&spec,
			test_helpers::new_db(),
			miner_service.clone(),
			IoChannel::disconnected(),
		).unwrap();
		let sync_provider = sync_provider();
		let external_miner = Arc::new(ExternalMiner::default());

		let eth_client = EthClient::new(
			&client,
			&snapshot_service,
			&sync_provider,
			&accounts,
			&miner_service,
			&external_miner,
			EthClientOptions {
				pending_nonce_from_queue: false,
				allow_pending_receipt_query: true,
				send_block_number_in_get_work: true,
				gas_price_percentile: 50,
				allow_experimental_rpcs: true,
				allow_missing_blocks: false
			},
		);

		let reservations = Arc::new(Mutex::new(nonce::Reservations::new(runtime.executor())));

		let dispatcher = FullDispatcher::new(client.clone(), miner_service.clone(), reservations, 50);
		let signer = Arc::new(dispatch::Signer::new(account_provider.clone())) as _;
		let eth_sign = SigningUnsafeClient::new(
			&signer,
			dispatcher,
		);

		let mut handler = IoHandler::default();
		handler.extend_with(eth_client.to_delegate());
		handler.extend_with(eth_sign.to_delegate());

		EthTester {
			_miner: miner_service,
			_runtime: runtime,
			_snapshot: snapshot_service,
			accounts: account_provider,
			client: client,
			handler: handler,
		}
	}
}

#[test]
fn harness_works() {
	let chain: BlockChain = extract_chain!("BlockchainTests/bcWalletTest/wallet2outOf3txs");
	let _ = EthTester::from_chain(&chain);
}

#[test]
fn eth_get_balance() {
	let chain = extract_chain!("BlockchainTests/bcWalletTest/wallet2outOf3txs");
	let tester = EthTester::from_chain(&chain);
	// final account state
	let req_latest = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBalance",
		"params": ["0xaaaf5374fce5edbc8e2a8697c15331677e6ebaaa", "latest"],
		"id": 1
	}"#;
	let res_latest = r#"{"jsonrpc":"2.0","result":"0x9","id":1}"#.to_owned();
	assert_eq!(tester.handler.handle_request_sync(req_latest).unwrap(), res_latest);

	// non-existant account
	let req_new_acc = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getBalance",
		"params": ["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
		"id": 3
	}"#;

	let res_new_acc = r#"{"jsonrpc":"2.0","result":"0x0","id":3}"#.to_owned();
	assert_eq!(tester.handler.handle_request_sync(req_new_acc).unwrap(), res_new_acc);
}

#[test]
fn eth_get_proof() {
	let chain = extract_chain!("BlockchainTests/bcWalletTest/wallet2outOf3txs");
	let tester = EthTester::from_chain(&chain);
	// final account state
	let req_latest = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getProof",
		"params": ["0xaaaf5374fce5edbc8e2a8697c15331677e6ebaaa", [], "latest"],
		"id": 1
	}"#;

	let res_latest = r#","address":"0xaaaf5374fce5edbc8e2a8697c15331677e6ebaaa","balance":"0x9","codeHash":"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470","nonce":"0x0","storageHash":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","storageProof":[]},"id":1}"#.to_owned();
    assert!(tester.handler.handle_request_sync(req_latest).unwrap().to_string().ends_with(res_latest.as_str()));

	// non-existant account
	let req_new_acc = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getProof",
		"params": ["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",[],"latest"],
		"id": 3
	}"#;

	let res_new_acc = r#","address":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","balance":"0x0","codeHash":"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470","nonce":"0x0","storageHash":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","storageProof":[]},"id":3}"#.to_owned();
    assert!(tester.handler.handle_request_sync(req_new_acc).unwrap().to_string().ends_with(res_new_acc.as_str()));
}

#[test]
fn eth_block_number() {
	let chain = extract_chain!("BlockchainTests/bcGasPricerTest/RPC_API_Test");
	let tester = EthTester::from_chain(&chain);
	let req_number = r#"{
		"jsonrpc": "2.0",
		"method": "eth_blockNumber",
		"params": [],
		"id": 1
	}"#;

	let res_number = r#"{"jsonrpc":"2.0","result":"0x20","id":1}"#.to_owned();
	assert_eq!(tester.handler.handle_request_sync(req_number).unwrap(), res_number);
}

#[test]
fn eth_get_block() {
	let chain = extract_chain!("BlockchainTests/bcGasPricerTest/RPC_API_Test");
	let tester = EthTester::from_chain(&chain);
	let req_block = r#"{"method":"eth_getBlockByNumber","params":["0x0",false],"id":1,"jsonrpc":"2.0"}"#;

	let res_block = r#"{"jsonrpc":"2.0","result":{"author":"0x8888f1f195afa192cfee860698584c030f4c9db1","difficulty":"0x20000","extraData":"0x42","gasLimit":"0x1df5d44","gasUsed":"0x0","hash":"0xcded1bc807465a72e2d54697076ab858f28b15d4beaae8faa47339c8eee386a3","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x8888f1f195afa192cfee860698584c030f4c9db1","mixHash":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","nonce":"0x0102030405060708","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","sealFields":["0xa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","0x880102030405060708"],"sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","size":"0x200","stateRoot":"0x7dba07d6b448a186e9612e5f737d1c909dce473e53199901a302c00646d523c1","timestamp":"0x54c98c81","totalDifficulty":"0x20000","transactions":[],"transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","uncles":[]},"id":1}"#;
	assert_eq!(tester.handler.handle_request_sync(req_block).unwrap(), res_block);
}

#[test]
fn eth_get_raw_block() {
	let chain = extract_chain!("BlockchainTests/bcGasPricerTest/RPC_API_Test");
	let tester = EthTester::from_chain(&chain);

	// In the test data, each block contains a "rlp" field. These tests are run against those field values.
	// blocknumber 1
	let req_block = r#"{"method":"eth_getRawBlockByNumber","params":["0x1",false],"id":1,"jsonrpc":"2.0"}"#;
	let res_block = r#"{"jsonrpc":"2.0","result":"0xf90968f901fba0cded1bc807465a72e2d54697076ab858f28b15d4beaae8faa47339c8eee386a3a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0ee57559895449b8dbd0a096b2999cf97b517b645ec8db33c7f5934778672263ea03ccbb984a0a736604acae327d9b643f8e75c7931cb2c6ac10dab4226e2e4c5a3a0a2bd925fcbb8b1ec39612553b17c9265ab198f5af25cc564655114bf5a28c75db901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000083020000018401dee56e83078674845bbdf64480a02a0928c245db5c5d50ac3bfa8e36f284e68ce47d136eba0b7e7e1d9b32b6be5d8828609b1e25e1917af90766f907638001832fefd8800ab907155b5b610705806100106000396000f3006000357c010000000000000000000000000000000000000000000000000000000090048063102accc11461012c57806312a7b9141461013a5780631774e6461461014c5780631e26fd331461015d5780631f9030371461016e578063343a875d1461018057806338cc4831146101955780634e7ad367146101bd57806357cb2fc4146101cb57806365538c73146101e057806368895979146101ee57806376bc21d9146102005780639a19a9531461020e5780639dc2c8f51461021f578063a53b1c1e1461022d578063a67808571461023e578063b61c05031461024c578063c2b12a731461025a578063d2282dc51461026b578063e30081a01461027c578063e8beef5b1461028d578063f38b06001461029b578063f5b53e17146102a9578063fd408767146102bb57005b6101346104d6565b60006000f35b61014261039b565b8060005260206000f35b610157600435610326565b60006000f35b6101686004356102c9565b60006000f35b610176610442565b8060005260206000f35b6101886103d3565b8060ff1660005260206000f35b61019d610413565b8073ffffffffffffffffffffffffffffffffffffffff1660005260206000f35b6101c56104c5565b60006000f35b6101d36103b7565b8060000b60005260206000f35b6101e8610454565b60006000f35b6101f6610401565b8060005260206000f35b61020861051f565b60006000f35b6102196004356102e5565b60006000f35b610227610693565b60006000f35b610238600435610342565b60006000f35b610246610484565b60006000f35b610254610493565b60006000f35b61026560043561038d565b60006000f35b610276600435610350565b60006000f35b61028760043561035e565b60006000f35b6102956105b4565b60006000f35b6102a3610547565b60006000f35b6102b16103ef565b8060005260206000f35b6102c3610600565b60006000f35b80600060006101000a81548160ff021916908302179055505b50565b80600060016101000a81548160ff02191690837f01000000000000000000000000000000000000000000000000000000000000009081020402179055505b50565b80600060026101000a81548160ff021916908302179055505b50565b806001600050819055505b50565b806002600050819055505b50565b80600360006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908302179055505b50565b806004600050819055505b50565b6000600060009054906101000a900460ff1690506103b4565b90565b6000600060019054906101000a900460000b90506103d0565b90565b6000600060029054906101000a900460ff1690506103ec565b90565b600060016000505490506103fe565b90565b60006002600050549050610410565b90565b6000600360009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905061043f565b90565b60006004600050549050610451565b90565b7f65c9ac8011e286e89d02a269890f41d67ca2cc597b2c76c7c69321ff492be5806000602a81526020016000a15b565b6000602a81526020016000a05b565b60017f81933b308056e7e85668661dcd102b1f22795b4431f9cf4625794f381c271c6b6000602a81526020016000a25b565b60016000602a81526020016000a15b565b3373ffffffffffffffffffffffffffffffffffffffff1660017f0e216b62efbb97e751a2ce09f607048751720397ecfb9eef1e48a6644948985b6000602a81526020016000a35b565b3373ffffffffffffffffffffffffffffffffffffffff1660016000602a81526020016000a25b565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6001023373ffffffffffffffffffffffffffffffffffffffff1660017f317b31292193c2a4f561cc40a95ea0d97a2733f14af6d6d59522473e1f3ae65f6000602a81526020016000a45b565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6001023373ffffffffffffffffffffffffffffffffffffffff1660016000602a81526020016000a35b565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6001023373ffffffffffffffffffffffffffffffffffffffff1660017fd5f0a30e4be0c6be577a71eceb7464245a796a7e6a55c0d971837b250de05f4e60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe98152602001602a81526020016000a45b565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6001023373ffffffffffffffffffffffffffffffffffffffff16600160007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe98152602001602a81526020016000a35b561ca0e439aa8812c1c0a751b0931ea20c5a30cd54fe15cae883c59fd8107e04557679a058d025af99b538b778a47da8115c43d5cee564c3cc8d58eb972aaf80ea2c406ec0","id":1}"#;
	assert_eq!(tester.handler.handle_request_sync(req_block).unwrap(), res_block);

	// blocknumber 2
	let req_block = r#"{"method":"eth_getRawBlockByNumber","params":["0x2",false],"id":1,"jsonrpc":"2.0"}"#;
	let res_block = r#"{"jsonrpc":"2.0","result":"0xf90266f901faa03afbe9e94654329fa016ead8b86fec531c6dc8805b07b7fa312477153c5327e5a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a052cbd86e23f3cd03140f49302f32ace2583c5e046c91049eb10136266b932caca0f6f36662c7d5cd443067f551d9874f11a9dfc9c3cfd72388beb19e60b585938ca0e9111d31a5282e8d68d1beaf1821405a9716182e2b780a724e1e6b78c609c6f3b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000083020000028401de6db68253f0845bbdf76480a0245b103751f0c19c2928c59b8f34f0623af1739c7393f499175a107a26bb13bc88e94f58ed82520f45f866f86401018304cb2f946295ee1b4f6dd65047762f924ecd367c17eabf8f0a8412a7b9141ba0ed2e0f715eccaab4362c19c1cf35ad8031ab1cabe71ada3fe8b269fe9d726712a06691074f289f826d23c92808ae363959eb958fb7a91fc721875ece4958114c65c0","id":1}"#;
	assert_eq!(tester.handler.handle_request_sync(req_block).unwrap(), res_block);
}

#[test]
fn eth_get_block_by_hash() {
	let chain = extract_chain!("BlockchainTests/bcGasPricerTest/RPC_API_Test");
	let tester = EthTester::from_chain(&chain);

	// We're looking for block number 4 from "RPC_API_Test_Frontier"
	let req_block = r#"{"method":"eth_getBlockByHash","params":["0xaddb9e39795e9e041c936b88a2577802569f34afded0948707b074caa3163a87",false],"id":1,"jsonrpc":"2.0"}"#;

	let res_block = r#"{"jsonrpc":"2.0","result":{"author":"0x8888f1f195afa192cfee860698584c030f4c9db1","difficulty":"0x20080","extraData":"0x","gasLimit":"0x1dd7ea0","gasUsed":"0x5458","hash":"0xaddb9e39795e9e041c936b88a2577802569f34afded0948707b074caa3163a87","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x8888f1f195afa192cfee860698584c030f4c9db1","mixHash":"0x713b0b31f6e72d8cb7367eaf59447ea531f209fc80e6379edd9f8d3bb73931c4","nonce":"0x4534b406bc23b86d","number":"0x4","parentHash":"0x17567aa5995b703736e32972289d68af50543acc4d56d37e8ad1fea7252cac4a","receiptsRoot":"0x7ed8026cf72ed0e98e6fd53ab406e51ffd34397d9da0052494ff41376fda7b5f","sealFields":["0xa0713b0b31f6e72d8cb7367eaf59447ea531f209fc80e6379edd9f8d3bb73931c4","0x884534b406bc23b86d"],"sha3Uncles":"0xe588a44b3e320e72e70b32b531f3ac0d432e756120135ae8fe5fa10895196b40","size":"0x661","stateRoot":"0x68805721294e365020aca15ed56c360d9dc2cf03cbeff84c9b84b8aed023bfb5","timestamp":"0x5bbdf772","totalDifficulty":"0xa00c0","transactions":["0xb094b9dc356dbb8b256402c6d5709288066ad6a372c90c9c516f14277545fd58"],"transactionsRoot":"0x97a593d8d7e15b57f5c6bb25bc6c325463ef99f874bc08a78656c3ab5cb23262","uncles":["0x86b48f5186c4b0882d3dca7977aa37840008832ef092f8ef797019dc74bfa8c7","0x2da9d062c11d536f0f1cc2a4e0111597c79926958d0fc26ae1a2d07d1a3bf47d"]},"id":1}"#;
	assert_eq!(tester.handler.handle_request_sync(req_block).unwrap(), res_block);
}

// a frontier-like test with an expanded gas limit and balance on known account.
const TRANSACTION_COUNT_SPEC: &'static [u8] = br#"{
	"name": "Frontier (Test)",
	"engine": {
		"Ethash": {
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"blockReward": "0x4563918244F40000",
				"durationLimit": "0x0d",
				"homesteadTransition": "0xffffffffffffffff",
				"daoHardforkTransition": "0xffffffffffffffff",
				"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
				"daoHardforkAccounts": []
			}
		}
	},
	"params": {
		"gasLimitBoundDivisor": "0x0400",
		"registrar" : "0xc6d9d2cd449a754c494264e1809c50e34d64562b",
		"accountStartNonce": "0x00",
		"maximumExtraDataSize": "0x20",
		"minGasLimit": "0x50000",
		"networkID" : "0x1"
	},
	"genesis": {
		"seal": {
			"ethereum": {
				"nonce": "0x0000000000000042",
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
			}
		},
		"difficulty": "0x400000000",
		"author": "0x0000000000000000000000000000000000000000",
		"timestamp": "0x00",
		"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
		"gasLimit": "0x50000"
	},
	"accounts": {
		"0000000000000000000000000000000000000001": { "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
		"0000000000000000000000000000000000000002": { "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
		"0000000000000000000000000000000000000003": { "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
		"0000000000000000000000000000000000000004": { "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
		"faa34835af5c2ea724333018a515fbb7d5bc0b33": { "balance": "10000000000000", "nonce": "0" }
	}
}
"#;

const POSITIVE_NONCE_SPEC: &'static [u8] = br#"{
	"name": "Frontier (Test)",
	"engine": {
		"Ethash": {
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"blockReward": "0x4563918244F40000",
				"durationLimit": "0x0d",
				"homesteadTransition": "0xffffffffffffffff",
				"daoHardforkTransition": "0xffffffffffffffff",
				"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
				"daoHardforkAccounts": []
			}
		}
	},
	"params": {
		"gasLimitBoundDivisor": "0x0400",
		"registrar" : "0xc6d9d2cd449a754c494264e1809c50e34d64562b",
		"accountStartNonce": "0x0100",
		"maximumExtraDataSize": "0x20",
		"minGasLimit": "0x50000",
		"networkID" : "0x1"
	},
	"genesis": {
		"seal": {
			"ethereum": {
				"nonce": "0x0000000000000042",
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
			}
		},
		"difficulty": "0x400000000",
		"author": "0x0000000000000000000000000000000000000000",
		"timestamp": "0x00",
		"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
		"gasLimit": "0x50000"
	},
	"accounts": {
		"0000000000000000000000000000000000000001": { "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
		"0000000000000000000000000000000000000002": { "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
		"0000000000000000000000000000000000000003": { "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
		"0000000000000000000000000000000000000004": { "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
		"faa34835af5c2ea724333018a515fbb7d5bc0b33": { "balance": "10000000000000", "nonce": "0" }
	}
}
"#;

#[test]
fn eth_transaction_count() {
	let secret = "8a283037bb19c4fed7b1c569e40c7dcff366165eb869110a1b11532963eb9cb2".parse().unwrap();
	let tester = EthTester::from_spec(Spec::load(&env::temp_dir(), TRANSACTION_COUNT_SPEC).expect("invalid chain spec"));
	let address = tester.accounts.insert_account(secret, &"".into()).unwrap();
	tester.accounts.unlock_account_permanently(address, "".into()).unwrap();

	let req_before = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": [""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"", "latest"],
		"id": 15
	}"#;

	let res_before = r#"{"jsonrpc":"2.0","result":"0x0","id":15}"#;

	assert_eq!(tester.handler.handle_request_sync(&req_before).unwrap(), res_before);

	let req_send_trans = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x30000",
			"gasPrice": "0x1",
			"value": "0x9184e72a"
		}],
		"id": 16
	}"#;

	// dispatch the transaction.
	tester.handler.handle_request_sync(&req_send_trans).unwrap();

	// we have submitted the transaction -- but this shouldn't be reflected in a "latest" query.
	let req_after_latest = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": [""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"", "latest"],
		"id": 17
	}"#;

	let res_after_latest = r#"{"jsonrpc":"2.0","result":"0x0","id":17}"#;

	assert_eq!(&tester.handler.handle_request_sync(&req_after_latest).unwrap(), res_after_latest);

	// the pending transactions should have been updated.
	let req_after_pending = r#"{
		"jsonrpc": "2.0",
		"method": "eth_getTransactionCount",
		"params": [""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"", "pending"],
		"id": 18
	}"#;

	let res_after_pending = r#"{"jsonrpc":"2.0","result":"0x1","id":18}"#;

	assert_eq!(&tester.handler.handle_request_sync(&req_after_pending).unwrap(), res_after_pending);
}

fn verify_transaction_counts(name: String, chain: BlockChain) {
	struct PanicHandler(String);
	impl Drop for PanicHandler {
		fn drop(&mut self) {
			if ::std::thread::panicking() {
				println!("Test failed: {}", self.0);
			}
		}
	}

	let _panic = PanicHandler(name);

	fn by_hash(hash: H256, count: usize, id: &mut usize) -> (String, String) {
		let req = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getBlockTransactionCountByHash",
			"params": [
				""#.to_owned() + format!("0x{:x}", hash).as_ref() + r#""
			],
			"id": "# + format!("{}", *id).as_ref() + r#"
		}"#;

		let res = r#"{"jsonrpc":"2.0","result":""#.to_owned()
			+ format!("0x{:x}", count).as_ref()
			+ r#"","id":"#
			+ format!("{}", *id).as_ref() + r#"}"#;
		*id += 1;
		(req, res)
	}

	fn by_number(num: u64, count: usize, id: &mut usize) -> (String, String) {
		let req = r#"{
			"jsonrpc": "2.0",
			"method": "eth_getBlockTransactionCountByNumber",
			"params": [
				"#.to_owned() + &::serde_json::to_string(&U256::from(num)).unwrap() + r#"
			],
			"id": "# + format!("{}", *id).as_ref() + r#"
		}"#;

		let res = r#"{"jsonrpc":"2.0","result":""#.to_owned()
			+ format!("0x{:x}", count).as_ref()
			+ r#"","id":"#
			+ format!("{}", *id).as_ref() + r#"}"#;
		*id += 1;
		(req, res)
	}

	let tester = EthTester::from_chain(&chain);

	let mut id = 1;
	for b in chain.blocks_rlp().into_iter().filter_map(|b| Unverified::from_rlp(b).ok()) {
		let count = b.transactions.len();

		let hash = b.header.hash();
		let number = b.header.number();

		let (req, res) = by_hash(hash, count, &mut id);
		assert_eq!(tester.handler.handle_request_sync(&req), Some(res));

		// uncles can share block numbers, so skip them.
		if tester.client.block_hash(BlockId::Number(number)) == Some(hash) {
			let (req, res) = by_number(number, count, &mut id);
			assert_eq!(tester.handler.handle_request_sync(&req), Some(res));
		}
	}
}

#[test]
fn starting_nonce_test() {
	let tester = EthTester::from_spec(Spec::load(&env::temp_dir(), POSITIVE_NONCE_SPEC).expect("invalid chain spec"));
	let address = Address::from(10);

	let sample = tester.handler.handle_request_sync(&(r#"
		{
			"jsonrpc": "2.0",
			"method": "eth_getTransactionCount",
			"params": [""#.to_owned() + format!("0x{:x}", address).as_ref() + r#"", "latest"],
			"id": 15
		}
		"#)
	).unwrap();

	assert_eq!(r#"{"jsonrpc":"2.0","result":"0x100","id":15}"#, &sample);
}

register_test!(eth_transaction_count_1, verify_transaction_counts, "BlockchainTests/bcWalletTest/wallet2outOf3txs");
register_test!(eth_transaction_count_2, verify_transaction_counts, "BlockchainTests/bcTotalDifficultyTest/sideChainWithMoreTransactions");
register_test!(eth_transaction_count_3, verify_transaction_counts, "BlockchainTests/bcGasPricerTest/RPC_API_Test");
