// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Parity-specific rpc implementation.
use std::sync::Arc;
use std::str::FromStr;
use std::collections::{BTreeMap, HashSet};

use ethereum_types::Address;
use version::version_data;

use crypto::DEFAULT_MAC;
use ethkey::{crypto::ecies, Brain, Generator};
use ethstore::random_phrase;
use sync::{SyncProvider, ManageNetwork};
use ethcore::account_provider::AccountProvider;
use ethcore::client::{BlockChainClient, StateClient, Call};
use ethcore::ids::BlockId;
use ethcore::miner::{self, MinerService};
use ethcore::state::StateInfo;
use ethcore_logger::RotatingLogger;
use updater::{Service as UpdateService};
use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::future;
use jsonrpc_macros::Trailing;
use v1::helpers::{self, errors, fake_sign, ipfs, SigningQueue, SignerService, NetworkSettings};
use v1::metadata::Metadata;
use v1::traits::Parity;
use v1::types::{
	Bytes, U256, U64, H160, H256, H512, CallRequest,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, DappId, ChainStatus,
	AccountInfo, HwAccountInfo, RichHeader,
	block_number_to_id, RichBlock, Block
};
use Host;

/// Parity implementation.
pub struct ParityClient<C, M, U> {
	client: Arc<C>,
	miner: Arc<M>,
	updater: Arc<U>,
	sync: Arc<SyncProvider>,
	net: Arc<ManageNetwork>,
	accounts: Arc<AccountProvider>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	signer: Option<Arc<SignerService>>,
	ws_address: Option<Host>,
}

impl<C, M, U> ParityClient<C, M, U> where
	C: BlockChainClient,
{
	/// Creates new `ParityClient`.
	pub fn new(
		client: Arc<C>,
		miner: Arc<M>,
		sync: Arc<SyncProvider>,
		updater: Arc<U>,
		net: Arc<ManageNetwork>,
		accounts: Arc<AccountProvider>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>,
		ws_address: Option<Host>,
	) -> Self {
		ParityClient {
			client,
			miner,
			sync,
			updater,
			net,
			accounts,
			logger,
			settings,
			signer,
			ws_address,
		}
	}
}

impl<C, M, U, S> Parity for ParityClient<C, M, U> where
	S: StateInfo + 'static,
	C: miner::BlockChainClient + BlockChainClient + StateClient<State=S> + Call<State=S> + 'static,
	M: MinerService<State=S> + 'static,
	U: UpdateService + 'static,
{
	type Metadata = Metadata;

	fn rich_block(&self, id: BlockNumberOrId, include_txs: bool) -> Result<Option<RichBlock>> {
		let client = &self.client;

		let client_query = |id| (client.block(id), client.block_total_difficulty(id), client.block_extra_info(id), false);

		let (block, difficulty, extra, is_pending) = match id {
			BlockNumberOrId::Number(BlockNumber::Pending) => {
				let info = self.client.chain_info();
				match self.miner.pending_block(info.best_block_number) {
					Some(pending_block) => {
						warn!("`Pending` is deprecated and may be removed in future versions.");

						let difficulty = {
							let latest_difficulty = self.client.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed");
							let pending_difficulty = self.miner.pending_block_header(info.best_block_number).map(|header| *header.difficulty());

							if let Some(difficulty) = pending_difficulty {
								difficulty + latest_difficulty
							} else {
								latest_difficulty
							}
						};

						let extra = self.client.engine().extra_info(&pending_block.header);

						(Some(encoded::Block::new(pending_block.rlp_bytes())), Some(difficulty), Some(extra), true)
					},
					None => {
						warn!("`Pending` is deprecated and may be removed in future versions. Falling back to `Latest`");
						client_query(BlockId::Latest)
					}
				}
			},

			BlockNumberOrId::Number(num) => {
				let id = match num {
					BlockNumber::Latest => BlockId::Latest,
					BlockNumber::Earliest => BlockId::Earliest,
					BlockNumber::Num(n) => BlockId::Number(n),
					BlockNumber::Pending => unreachable!() // Already covered
				};

				client_query(id)
			},

			BlockNumberOrId::Id(id) => client_query(id),
		};

		match (block, difficulty) {
			(Some(block), Some(total_difficulty)) => {
				let view = block.header_view();
				Ok(Some(RichBlock {
					inner: Block {
						hash: match is_pending {
							true => None,
							false => Some(view.hash().into()),
						},
						size: Some(block.rlp().as_raw().len().into()),
						parent_hash: view.parent_hash().into(),
						uncles_hash: view.uncles_hash().into(),
						author: view.author().into(),
						miner: view.author().into(),
						state_root: view.state_root().into(),
						transactions_root: view.transactions_root().into(),
						receipts_root: view.receipts_root().into(),
						number: match is_pending {
							true => None,
							false => Some(view.number().into()),
						},
						gas_used: view.gas_used().into(),
						gas_limit: view.gas_limit().into(),
						logs_bloom: match is_pending {
							true => None,
							false => Some(view.log_bloom().into()),
						},
						timestamp: view.timestamp().into(),
						difficulty: view.difficulty().into(),
						total_difficulty: Some(total_difficulty.into()),
						seal_fields: view.seal().into_iter().map(Into::into).collect(),
						uncles: block.uncle_hashes().into_iter().map(Into::into).collect(),
						transactions: match include_txs {
							true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(|t| Transaction::from_localized(t)).collect()),
							false => BlockTransactions::Hashes(block.transaction_hashes().into_iter().map(Into::into).collect()),
						},
						extra_data: Bytes::new(view.extra_data()),
					},
					extra_info: extra.expect(EXTRA_INFO_PROOF),
				}))
			},
			_ => Ok(None)
		}
	}

	fn accounts_info(&self, dapp: Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>> {
		let dapp = dapp.unwrap_or_default();

		let dapp_accounts = self.accounts
			.note_dapp_used(dapp.clone().into())
			.and_then(|_| self.accounts.dapp_addresses(dapp.into()))
			.map_err(|e| errors::account("Could not fetch accounts.", e))?
			.into_iter().collect::<HashSet<_>>();

		let info = self.accounts.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = self.accounts.addresses_info();

		Ok(info
			.into_iter()
			.chain(other.into_iter())
			.filter(|&(ref a, _)| dapp_accounts.contains(a))
			.map(|(a, v)| (H160::from(a), AccountInfo { name: v.name }))
			.collect()
		)
	}

	fn hardware_accounts_info(&self) -> Result<BTreeMap<H160, HwAccountInfo>> {
		let info = self.accounts.hardware_accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		Ok(info
			.into_iter()
			.map(|(a, v)| (H160::from(a), HwAccountInfo { name: v.name, manufacturer: v.meta }))
			.collect()
		)
	}

	fn locked_hardware_accounts_info(&self) -> Result<Vec<String>> {
		self.accounts.locked_hardware_accounts().map_err(|e| errors::account("Error communicating with hardware wallet.", e))
	}

	fn default_account(&self, meta: Self::Metadata) -> Result<H160> {
		let dapp_id = meta.dapp_id();

		Ok(self.accounts
			.dapp_default_address(dapp_id.into())
			.map(Into::into)
			.ok()
			.unwrap_or_default())
	}

	fn transactions_limit(&self) -> Result<usize> {
		Ok(self.miner.queue_status().limits.max_count)
	}

	fn min_gas_price(&self) -> Result<U256> {
		Ok(self.miner.queue_status().options.minimal_gas_price.into())
	}

	fn extra_data(&self) -> Result<Bytes> {
		Ok(Bytes::new(self.miner.authoring_params().extra_data))
	}

	fn gas_floor_target(&self) -> Result<U256> {
		Ok(U256::from(self.miner.authoring_params().gas_range_target.0))
	}

	fn gas_ceil_target(&self) -> Result<U256> {
		Ok(U256::from(self.miner.authoring_params().gas_range_target.1))
	}

	fn dev_logs(&self) -> Result<Vec<String>> {
		let logs = self.logger.logs();
		Ok(logs.as_slice().to_owned())
	}

	fn dev_logs_levels(&self) -> Result<String> {
		Ok(self.logger.levels().to_owned())
	}

	fn net_chain(&self) -> Result<String> {
		Ok(self.settings.chain.clone())
	}

	fn chain_id(&self) -> Result<Option<U64>> {
		Ok(self.client.signing_chain_id().map(U64::from))
	}

	fn chain(&self) -> Result<String> {
		Ok(self.client.spec_name())
	}

	fn net_peers(&self) -> Result<Peers> {
		let sync_status = self.sync.status();
		let num_peers_range = self.net.num_peers_range();
		debug_assert!(num_peers_range.end > num_peers_range.start);
		let peers = self.sync.peers().into_iter().map(Into::into).collect();

		Ok(Peers {
			active: sync_status.num_active_peers,
			connected: sync_status.num_peers,
			max: sync_status.current_max_peers(num_peers_range.start, num_peers_range.end - 1),
			peers: peers
		})
	}

	fn net_port(&self) -> Result<u16> {
		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String> {
		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>> {
		Ok(
			self.client
				.additional_params()
				.get("registrar")
				.and_then(|s| Address::from_str(s).ok())
				.map(|s| H160::from(s))
		)
	}

	fn rpc_settings(&self) -> Result<RpcSettings> {
		Ok(RpcSettings {
			enabled: self.settings.rpc_enabled,
			interface: self.settings.rpc_interface.clone(),
			port: self.settings.rpc_port as u64,
		})
	}

	fn default_extra_data(&self) -> Result<Bytes> {
		Ok(Bytes::new(version_data()))
	}

	fn gas_price_histogram(&self) -> BoxFuture<Histogram> {
		Box::new(future::done(self.client
			.gas_price_corpus(100)
			.histogram(10)
			.ok_or_else(errors::not_enough_data)
			.map(Into::into)
		))
	}

	fn unsigned_transactions_count(&self) -> Result<usize> {
		match self.signer {
			None => Err(errors::signer_disabled()),
			Some(ref signer) => Ok(signer.len()),
		}
	}

	fn generate_secret_phrase(&self) -> Result<String> {
		Ok(random_phrase(12))
	}

	fn phrase_to_address(&self, phrase: String) -> Result<H160> {
		Ok(Brain::new(phrase).generate().unwrap().address().into())
	}

	fn list_accounts(&self, count: u64, after: Option<H160>, block_number: Trailing<BlockNumber>) -> Result<Option<Vec<H160>>> {
		let number = match block_number.unwrap_or_default() {
			BlockNumber::Pending => {
				warn!("BlockNumber::Pending is unsupported");
				return Ok(None);
			},

			num => block_number_to_id(num)
		};

		Ok(self.client
			.list_accounts(number, after.map(Into::into).as_ref(), count)
			.map(|a| a.into_iter().map(Into::into).collect()))
	}

	fn list_storage_keys(&self, address: H160, count: u64, after: Option<H256>, block_number: Trailing<BlockNumber>) -> Result<Option<Vec<H256>>> {
		let number = match block_number.unwrap_or_default() {
			BlockNumber::Pending => {
				warn!("BlockNumber::Pending is unsupported");
				return Ok(None);
			},

			num => block_number_to_id(num)
		};

		Ok(self.client
			.list_storage(number, &address.into(), after.map(Into::into).as_ref(), count)
			.map(|a| a.into_iter().map(Into::into).collect()))
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes> {
		ecies::encrypt(&key.into(), &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption)
			.map(Into::into)
	}

	fn pending_transactions(&self, limit: Trailing<usize>) -> Result<Vec<Transaction>> {
		let ready_transactions = self.miner.ready_transactions(
			&*self.client,
			limit.unwrap_or_else(usize::max_value),
			miner::PendingOrdering::Priority,
		);

		Ok(ready_transactions
			.into_iter()
			.map(|t| Transaction::from_pending(t.pending().clone()))
			.collect()
		)
	}

	fn all_transactions(&self) -> Result<Vec<Transaction>> {
		let all_transactions = self.miner.queued_transactions();

		Ok(all_transactions
			.into_iter()
			.map(|t| Transaction::from_pending(t.pending().clone()))
			.collect()
		)
	}

	fn future_transactions(&self) -> Result<Vec<Transaction>> {
		Err(errors::deprecated("Use `parity_allTransaction` instead."))
	}

	fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>> {
		let stats = self.sync.transactions_stats();
		Ok(stats.into_iter()
			.map(|(hash, stats)| (hash.into(), stats.into()))
			.collect()
		)
	}

	fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>> {
		let transactions = self.miner.local_transactions();
		Ok(transactions
			.into_iter()
			.map(|(hash, status)| (hash.into(), LocalTransactionStatus::from(status)))
			.collect()
		)
	}

	fn ws_url(&self) -> Result<String> {
		helpers::to_url(&self.ws_address)
			.ok_or_else(|| errors::ws_disabled())
	}

	fn next_nonce(&self, address: H160) -> BoxFuture<U256> {
		let address: Address = address.into();

		Box::new(future::ok(self.miner.next_nonce(&*self.client, &address).into()))
	}

	fn mode(&self) -> Result<String> {
		Ok(self.client.mode().to_string())
	}

	fn enode(&self) -> Result<String> {
		self.sync.enode().ok_or_else(errors::network_disabled)
	}

	fn consensus_capability(&self) -> Result<ConsensusCapability> {
		Ok(self.updater.capability().into())
	}

	fn version_info(&self) -> Result<VersionInfo> {
		Ok(self.updater.version_info().into())
	}

	fn releases_info(&self) -> Result<Option<OperationsInfo>> {
		Ok(self.updater.info().map(Into::into))
	}

	fn chain_status(&self) -> Result<ChainStatus> {
		let chain_info = self.client.chain_info();

		let gap = chain_info.ancient_block_number.map(|x| U256::from(x + 1))
			.and_then(|first| chain_info.first_block_number.map(|last| (first, U256::from(last))));

		Ok(ChainStatus {
			block_gap: gap.map(|(x, y)| (x.into(), y.into())),
		})
	}

	fn node_kind(&self) -> Result<::v1::types::NodeKind> {
		use ::v1::types::{NodeKind, Availability, Capability};

		Ok(NodeKind {
			availability: Availability::Personal,
			capability: Capability::Full,
		})
	}

	fn block_header(&self, number: Trailing<BlockNumber>) -> BoxFuture<RichHeader> {
		const EXTRA_INFO_PROOF: &str = "Object exists in blockchain (fetched earlier), extra_info is always available if object exists; qed";
		let number = number.unwrap_or_default();

		let (header, extra) = if number == BlockNumber::Pending {
			let info = self.client.chain_info();
			let header = try_bf!(self.miner.pending_block_header(info.best_block_number).ok_or(errors::unknown_block()));

			(header.encoded(), None)
		} else {
			let id = match number {
				BlockNumber::Num(num) => BlockId::Number(num),
				BlockNumber::Earliest => BlockId::Earliest,
				BlockNumber::Latest => BlockId::Latest,
				BlockNumber::Pending => unreachable!(), // Already covered
			};

			let header = try_bf!(self.client.block_header(id.clone()).ok_or(errors::unknown_block()));
			let info = self.client.block_extra_info(id).expect(EXTRA_INFO_PROOF);

			(header, Some(info))
		};

		Box::new(future::ok(RichHeader {
			inner: header.into(),
			extra_info: extra.unwrap_or_default(),
		}))
	}

		fn raw_block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>> {
			Box::new(future::done(self.rich_block(num.into(), include_txs)))
		}

		fn submit_block(&self, block: RichBlock) -> Result<RpcH256> {
			// TODO: Re-implement rlp_bytes for rpc/v1/src/types/block.rb 
			//       or reuse the rlp_bytes from ethcore/src/block.rb?
			rlp_encoded_block = block.rlp_bytes()
			Rlp::new(&rlp_encoded_block.into_vec()).as_val()
				.map_err(errors::rlp)
				.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))
				.and_then(|signed_transaction| {
					FullDispatcher::dispatch_transaction(
						&*self.client,
						&*self.miner,
						signed_transaction.into(),
						false
					)
			})
			.map(Into::into)
		}


	fn ipfs_cid(&self, content: Bytes) -> Result<String> {
		ipfs::cid(content)
	}

	fn call(&self, meta: Self::Metadata, requests: Vec<CallRequest>, num: Trailing<BlockNumber>) -> Result<Vec<Bytes>> {
		let requests = requests
			.into_iter()
			.map(|request| Ok((
				fake_sign::sign_call(request.into(), meta.is_dapp())?,
				Default::default()
			)))
			.collect::<Result<Vec<_>>>()?;

		let num = num.unwrap_or_default();

		let (mut state, header) = if num == BlockNumber::Pending {
			let info = self.client.chain_info();
			let state = self.miner.pending_state(info.best_block_number).ok_or(errors::state_pruned())?;
			let header = self.miner.pending_block_header(info.best_block_number).ok_or(errors::state_pruned())?;

			(state, header)
		} else {
			let id = match num {
				BlockNumber::Num(num) => BlockId::Number(num),
				BlockNumber::Earliest => BlockId::Earliest,
				BlockNumber::Latest => BlockId::Latest,
				BlockNumber::Pending => unreachable!(), // Already covered
			};

			let state = self.client.state_at(id).ok_or(errors::state_pruned())?;
			let header = self.client.block_header(id).ok_or(errors::state_pruned())?.decode().map_err(errors::decode)?;

			(state, header)
		};

		self.client.call_many(&requests, &mut state, &header)
				.map(|res| res.into_iter().map(|res| res.output.into()).collect())
				.map_err(errors::call)
	}
}
