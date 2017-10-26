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

//! Client-side stratum job dispatcher and mining notifier handler

use ethcore_stratum::{
	JobDispatcher, PushWorkHandler,
	Stratum as StratumService, Error as StratumServiceError,
};

use std::sync::{Arc, Weak};
use std::net::{SocketAddr, AddrParseError};
use std::fmt;

use bigint::prelude::U256;
use bigint::hash::{H64, H256, clean_0x};
use ethereum::ethash::Ethash;
use ethash::SeedHashCompute;
use parking_lot::Mutex;
use miner::{self, Miner, MinerService};
use client::Client;
use block::IsBlock;
use rlp::encode;

/// Configures stratum server options.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
	/// Working directory
	pub io_path: String,
	/// Network address
	pub listen_addr: String,
	/// Port
	pub port: u16,
	/// Secret for peers
	pub secret: Option<H256>,
}

struct SubmitPayload {
	nonce: H64,
	pow_hash: H256,
	mix_hash: H256,
}

impl SubmitPayload {
	fn from_args(payload: Vec<String>) -> Result<Self, PayloadError> {
		if payload.len() != 3 {
			return Err(PayloadError::ArgumentsAmountUnexpected(payload.len()));
		}

		let nonce = match clean_0x(&payload[0]).parse::<H64>() {
			Ok(nonce) => nonce,
			Err(e) => {
				warn!(target: "stratum", "submit_work ({}): invalid nonce ({:?})", &payload[0], e);
				return Err(PayloadError::InvalidNonce(payload[0].clone()))
			}
		};

		let pow_hash = match clean_0x(&payload[1]).parse::<H256>() {
			Ok(pow_hash) => pow_hash,
			Err(e) => {
				warn!(target: "stratum", "submit_work ({}): invalid hash ({:?})", &payload[1], e);
				return Err(PayloadError::InvalidPowHash(payload[1].clone()));
			}
		};

		let mix_hash = match clean_0x(&payload[2]).parse::<H256>() {
			Ok(mix_hash) => mix_hash,
			Err(e) => {
				warn!(target: "stratum", "submit_work ({}): invalid mix-hash ({:?})",  &payload[2], e);
				return Err(PayloadError::InvalidMixHash(payload[2].clone()));
			}
		};

		Ok(SubmitPayload {
			nonce: nonce,
			pow_hash: pow_hash,
			mix_hash: mix_hash,
		})
	}
}

#[derive(Debug)]
enum PayloadError {
	ArgumentsAmountUnexpected(usize),
	InvalidNonce(String),
	InvalidPowHash(String),
	InvalidMixHash(String),
}

impl fmt::Display for PayloadError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

/// Job dispatcher for stratum service
pub struct StratumJobDispatcher {
	seed_compute: Mutex<SeedHashCompute>,
	client: Weak<Client>,
	miner: Weak<Miner>,
}


impl JobDispatcher for StratumJobDispatcher {
	fn initial(&self) -> Option<String> {
		// initial payload may contain additional data, not in this case
		self.job()
	}

	fn job(&self) -> Option<String> {
		self.with_core(|client, miner| miner.map_sealing_work(&*client, |b| {
				let pow_hash = b.hash();
				let number = b.block().header().number();
				let difficulty = b.block().header().difficulty();

				self.payload(pow_hash, *difficulty, number)
			})
		)
	}

	fn submit(&self, payload: Vec<String>) -> Result<(), StratumServiceError> {
		let payload = SubmitPayload::from_args(payload).map_err(|e|
			StratumServiceError::Dispatch(e.to_string())
		)?;

		trace!(
			target: "stratum",
			"submit_work: Decoded: nonce={}, pow_hash={}, mix_hash={}",
			payload.nonce,
			payload.pow_hash,
			payload.mix_hash,
		);

		self.with_core_result(|client, miner| {
			let seal = vec![encode(&payload.mix_hash).into_vec(), encode(&payload.nonce).into_vec()];
			match miner.submit_seal(&*client, payload.pow_hash, seal) {
				Ok(_) => Ok(()),
				Err(e) => {
					warn!(target: "stratum", "submit_seal error: {:?}", e);
					Err(StratumServiceError::Dispatch(e.to_string()))
				}
			}
		})
	}
}

impl StratumJobDispatcher {
	/// New stratum job dispatcher given the miner and client
	fn new(miner: Weak<Miner>, client: Weak<Client>) -> StratumJobDispatcher {
		StratumJobDispatcher {
			seed_compute: Mutex::new(SeedHashCompute::new()),
			client: client,
			miner: miner,
		}
	}

	/// Serializes payload for stratum service
	fn payload(&self, pow_hash: H256, difficulty: U256, number: u64) -> String {
		// TODO: move this to engine
		let target = Ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().hash_block_number(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		format!(
			r#"["0x", "0x{}","0x{}","0x{}","0x{:x}"]"#,
			pow_hash.hex(), seed_hash.hex(), target.hex(), number
		)
	}

	fn with_core<F, R>(&self, f: F) -> Option<R> where F: Fn(Arc<Client>, Arc<Miner>) -> Option<R> {
		self.client.upgrade().and_then(|client| self.miner.upgrade().and_then(|miner| (f)(client, miner)))
	}

	fn with_core_result<F>(&self, f: F) -> Result<(), StratumServiceError> where F: Fn(Arc<Client>, Arc<Miner>) -> Result<(), StratumServiceError> {
		match (self.client.upgrade(), self.miner.upgrade()) {
			(Some(client), Some(miner)) => f(client, miner),
			_ => Ok(()),
		}
	}
}

/// Wrapper for dedicated stratum service
pub struct Stratum {
	dispatcher: Arc<StratumJobDispatcher>,
	service: Arc<StratumService>,
}

#[derive(Debug)]
/// Stratum error
pub enum Error {
	/// IPC sockets error
	Service(StratumServiceError),
	/// Invalid network address
	Address(AddrParseError),
}

impl From<StratumServiceError> for Error {
	fn from(service_err: StratumServiceError) -> Error { Error::Service(service_err) }
}

impl From<AddrParseError> for Error {
	fn from(err: AddrParseError) -> Error { Error::Address(err) }
}

impl super::work_notify::NotifyWork for Stratum {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		trace!(target: "stratum", "Notify work");

		self.service.push_work_all(
			self.dispatcher.payload(pow_hash, difficulty, number)
		).unwrap_or_else(
			|e| warn!(target: "stratum", "Error while pushing work: {:?}", e)
		);
	}
}

impl Stratum {

	/// New stratum job dispatcher, given the miner, client and dedicated stratum service
	pub fn start(options: &Options, miner: Weak<Miner>, client: Weak<Client>) -> Result<Stratum, Error> {
		use std::net::IpAddr;

		let dispatcher = Arc::new(StratumJobDispatcher::new(miner, client));

		let stratum_svc = StratumService::start(
			&SocketAddr::new(options.listen_addr.parse::<IpAddr>()?, options.port),
			dispatcher.clone(),
			options.secret.clone(),
		)?;

		Ok(Stratum {
			dispatcher: dispatcher,
			service: stratum_svc,
		})
	}

	/// Start STRATUM job dispatcher and register it in the miner
	pub fn register(cfg: &Options, miner: Arc<Miner>, client: Weak<Client>) -> Result<(), Error> {
		let stratum = miner::Stratum::start(cfg, Arc::downgrade(&miner.clone()), client)?;
		miner.push_notifier(Box::new(stratum) as Box<miner::NotifyWork>);
		Ok(())
	}
}
