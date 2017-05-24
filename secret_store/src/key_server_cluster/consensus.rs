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

use std::fmt::Debug;
use std::collections::{BTreeSet, BTreeMap};
use ethkey::Secret;
use key_server_cluster::{Error, NodeId};
use key_server_cluster::math;

#[derive(Debug, Clone)]
/// Consensus.
pub enum Consensus<T: Debug + Clone> {
	/// Consensus is currently establishing.
	Establishing(ConsensusCore),
	/// Consensus is established.
	Established(ConsensusCore),
	/// Consensus nodes are currently doing their job.
	Active(ActiveConsensus<T>),
	/// All consensus nodes have responded.
	Completed(ActiveConsensus<T>),
	/// Consensus is unreachable.
	Unreachable,
}

#[derive(Debug, Clone)]
/// Consensus core data.
pub struct ConsensusCore {
	/// Consensus threshold.
	threshold: usize,
	/// Nodes, which have been requested for participatining in consensus, but not yet responded.
	requested_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with reject to participation request.
	rejected_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with confirm to participation request.
	confirmed_nodes: BTreeSet<NodeId>,
}

#[derive(Debug, Clone)]
/// Active consensus (i.e. consensus with sent requests).
pub struct ActiveConsensus<T: Debug + Clone> {
	/// Consensus core data.
	core: ConsensusCore,
	/// Selection key.
	selection_key: Option<Secret>,
	/// Selected nodes.
	selected_nodes: BTreeSet<NodeId>,
	/// Active job requests to confirmed nodes.
	active_requests: BTreeSet<NodeId>,
	/// Confirmed nodes responses.
	responses: BTreeMap<NodeId, T>,
}

impl<T> Consensus<T> where T: Debug + Clone {
	/// Create new consensus.
	pub fn new(threshold: usize, nodes: BTreeSet<NodeId>) -> Result<Self, Error> {
		if nodes.len() < threshold + 1 {
			return Err(Error::InvalidThreshold);
		}

		Ok(Consensus::Establishing(ConsensusCore {
			threshold: threshold,
			requested_nodes: nodes,
			rejected_nodes: BTreeSet::new(),
			confirmed_nodes: BTreeSet::new(),
		}))
	}

	#[cfg(test)]
	/// Return consensus core reference.
	pub fn core(&self) -> Option<&ConsensusCore> {
		match *self {
			Consensus::Establishing(ref consensus) | Consensus::Established(ref consensus) => Some(&consensus),
			Consensus::Active(ref consensus) | Consensus::Completed(ref consensus) => Some(&consensus.core),
			Consensus::Unreachable => None,
		}
	}

	/// Is consenus established.
	pub fn is_established(&self) -> bool {
		match *self {
			Consensus::Established(_) => true,
			_ => false,
		}
	}

	/// Are consenus jobs completed.
	pub fn is_completed(&self) -> bool {
		match *self {
			Consensus::Completed(_) => true,
			_ => false,
		}
	}

	/// Is consensus unreachable.
	pub fn is_unreachable(&self) -> bool {
		match *self {
			Consensus::Unreachable => true,
			_ => false,
		}
	}

	/// When node responds to join offer.
	pub fn offer_response(&mut self, node: &NodeId, is_accepted: bool) -> Result<(), Error> {
		if is_accepted {
			self.accept_offer(node)
		} else {
			self.reject_offer(node)
		}
	}

	/// When node has accepted join offer.
	pub fn accept_offer(&mut self, node: &NodeId) -> Result<(), Error> {
		let established_consensus = match *self {
			Consensus::Establishing(ref mut consensus) => {
				consensus.accept_offer(node)?;
				if consensus.confirmed_nodes.len() != consensus.threshold + 1 {
					return Ok(());
				}

				consensus.clone()
			},
			Consensus::Established(ref mut consensus) => return consensus.accept_offer(node),
			Consensus::Active(ref mut consensus) | Consensus::Completed(ref mut consensus) =>
				return consensus.core.accept_offer(node),
			Consensus::Unreachable => return Err(Error::InvalidStateForRequest),
		};

		*self = Consensus::Established(established_consensus);
		Ok(())
	}

	/// When node has rejected join offer.
	pub fn reject_offer(&mut self, node: &NodeId) -> Result<(), Error> {
		match *self {
			Consensus::Establishing(ref mut consensus) => {
				consensus.reject_offer(node)?;
				if consensus.requested_nodes.len() + consensus.confirmed_nodes.len() >= consensus.threshold + 1 {
					return Ok(());
				}

				// else fall through
			},
			Consensus::Established(ref mut consensus) => return consensus.reject_offer(node),
			Consensus::Active(ref mut consensus) | Consensus::Completed(ref mut consensus) =>
				return consensus.core.reject_offer(node),
			_ => return Err(Error::InvalidStateForRequest),
		}

		*self = Consensus::Unreachable;
		Ok(())
	}

	/// When starting/restarting requesting consensus nodes to do their job.
	pub fn activate(&mut self) -> Result<(), Error> {
		let active_consensus = match *self {
			Consensus::Established(ref established_consensus) => ActiveConsensus::new(established_consensus.clone()),
			Consensus::Active(ref active_consensus) => ActiveConsensus::new(active_consensus.core.clone()),
			_ => return Err(Error::InvalidStateForRequest),
		};

		*self = Consensus::Active(active_consensus);
		Ok(())
	}

	/// Select nodes for completing their jobs.
	pub fn select_nodes(&mut self) -> Result<(&Secret, &BTreeSet<NodeId>), Error> {
		match *self {
			Consensus::Active(ref mut consensus) => consensus.select_nodes(),
			_ => Err(Error::InvalidStateForRequest),
		}
	}

	/// Get nodes, select nodes for completing their jobs.
	pub fn selected_nodes(&self) -> Result<(&Secret, &BTreeSet<NodeId>), Error> {
		match *self {
			Consensus::Active(ref consensus) => consensus.selected_nodes(),
			_ => Err(Error::InvalidStateForRequest),
		}
	}

	/// When job request is sent to the node.
	pub fn job_request_sent(&mut self, node: &NodeId) -> Result<(), Error> {
		match *self {
			Consensus::Active(ref mut consensus) => consensus.job_request_sent(node),
			_ => Err(Error::InvalidStateForRequest),
		}
	}

	/// When job response is received from the node.
	pub fn job_response_received(&mut self, node: &NodeId, selection_key: &Secret, response: T) -> Result<(), Error> {
		let completed_consensus = match *self {
			Consensus::Active(ref mut consensus) => {
				consensus.job_response_received(node, selection_key, response)?;
				if consensus.responses.len() != consensus.core.threshold + 1 {
					return Ok(());
				}
				debug_assert!(consensus.active_requests.is_empty());

				// else fall through
				consensus.clone()
			},
			_ => return Err(Error::InvalidStateForRequest),
		};

		*self = Consensus::Completed(completed_consensus);
		Ok(())
	}

	#[cfg(test)]
	/// Return job reqeuests.
	pub fn job_requests(&self) -> Result<&BTreeSet<NodeId>, Error> {
		match *self {
			Consensus::Active(ref consensus) => consensus.job_requests(),
			_ => Err(Error::InvalidStateForRequest),
		}
	}

	/// Return job responses.
	pub fn job_responses(&self) -> Result<&BTreeMap<NodeId, T>, Error> {
		match *self {
			Consensus::Active(ref consensus) => consensus.job_responses(),
			Consensus::Completed(ref consensus) => consensus.job_responses(),
			_ => Err(Error::InvalidStateForRequest),
		}
	}

	/// When node is timeouted. Returns true if consensus restarted (i.e. caller must resend job requests).
	pub fn node_timeouted(&mut self, node: &NodeId) -> Result<bool, Error> {
		match *self {
			Consensus::Establishing(ref mut consensus) => {
				consensus.node_timeouted(node)?;
				if consensus.requested_nodes.len() + consensus.confirmed_nodes.len() >= consensus.threshold + 1 {
					return Ok(false);
				}

				// else fall through
			},
			Consensus::Established(ref mut consensus) => {
				consensus.node_timeouted(node)?;
				if consensus.requested_nodes.len() + consensus.confirmed_nodes.len() >= consensus.threshold + 1 {
					return Ok(false);
				}

				// else fall through
			}
			Consensus::Active(ref mut consensus) => {
				let is_restart_required = consensus.node_timeouted(node)?;
				let is_consensus_reachable = consensus.core.requested_nodes.len() + consensus.core.confirmed_nodes.len() >= consensus.core.threshold + 1;
				if is_consensus_reachable {
					if !is_restart_required {
						return Ok(false);
					}

					consensus.restart()?;
					return Ok(true);
				}

				// else fall through
			},
			Consensus::Completed(_) => return Ok(false),
			_ => return Err(Error::InvalidStateForRequest),
		}

		*self = Consensus::Unreachable;
		Err(Error::ConsensusUnreachable)
	}

	/// When session is timeouted. Consensus is always restarted (if active).
	pub fn session_timeouted(&mut self) -> Result<(), Error> {
		match *self {
			Consensus::Establishing(_) => (), // fall through
			Consensus::Established(_) => (), // fall through
			Consensus::Active(ref mut consensus) => {
				consensus.session_timeouted()?;
				if consensus.core.requested_nodes.len() + consensus.core.confirmed_nodes.len() >= consensus.core.threshold + 1 {
					return Ok(());
				}

				// else fall through
			},
			Consensus::Completed(_) | Consensus::Unreachable => return Ok(()),
		}

		*self = Consensus::Unreachable;
		Err(Error::ConsensusUnreachable)
	}
}

impl ConsensusCore {
	#[cfg(test)]
	/// Return rejected nodes list.
	pub fn rejected_nodes(&self) -> &BTreeSet<NodeId> {
		&self.rejected_nodes
	}

	/// When node has accepted join offer.
	pub fn accept_offer(&mut self, node: &NodeId) -> Result<(), Error> {
		if !self.requested_nodes.remove(node) {
			return Err(Error::InvalidStateForRequest);
		}

		self.confirmed_nodes.insert(node.clone());
		Ok(())
	}

	/// When node has rejected join offer.
	pub fn reject_offer(&mut self, node: &NodeId) -> Result<(), Error> {
		if !self.requested_nodes.remove(node) {
			return Err(Error::InvalidStateForRequest);
		}

		self.rejected_nodes.insert(node.clone());
		Ok(())
	}

	/// When node is timeouted.
	pub fn node_timeouted(&mut self, node: &NodeId) -> Result<(), Error> {
		if self.requested_nodes.remove(node) || self.confirmed_nodes.remove(node) {
			self.rejected_nodes.insert(node.clone());
		}
		Ok(())
	}
}

impl<T> ActiveConsensus<T> where T: Debug + Clone {
	/// Create new active consensus.
	pub fn new(core: ConsensusCore) -> Self {
		ActiveConsensus {
			core: core,
			selection_key: None,
			selected_nodes: BTreeSet::new(),
			active_requests: BTreeSet::new(),
			responses: BTreeMap::new(),
		}
	}

	/// Select nodes to make job.
	pub fn select_nodes(&mut self) -> Result<(&Secret, &BTreeSet<NodeId>), Error> {
		if !self.selected_nodes.is_empty() {
			return Err(Error::InvalidStateForRequest);
		}

		// TODO: possibly optimize by including this node in selected_nodes list
		self.selection_key = Some(math::generate_random_scalar()?);
		self.selected_nodes = self.core.confirmed_nodes.iter().cloned().take(self.core.threshold + 1).collect();
		Ok((self.selection_key.as_ref().expect("filled couple of lines above"), &self.selected_nodes))
	}

	/// Get nodes, selected nodes to make their job.
	pub fn selected_nodes(&self) -> Result<(&Secret, &BTreeSet<NodeId>), Error> {
		if self.selection_key.is_none() || self.selected_nodes.is_empty() {
			return Err(Error::InvalidStateForRequest);
		}

		Ok((self.selection_key.as_ref().expect("checked couple of lines above"), &self.selected_nodes))
	}

	/// When job request is sent to the node.
	pub fn job_request_sent(&mut self, node: &NodeId) -> Result<(), Error> {
		if !self.core.confirmed_nodes.contains(node) {
			return Err(Error::InvalidNodeForRequest);
		}
		if !self.selected_nodes.contains(node) {
			return Err(Error::InvalidNodeForRequest);
		}
		if !self.active_requests.insert(node.clone()) {
			return Err(Error::InvalidNodeForRequest);
		}

		debug_assert!(self.active_requests.len() <= self.core.threshold + 1);
		Ok(())
	}

	/// When job response is received from the node.
	pub fn job_response_received(&mut self, node: &NodeId, selection_key: &Secret, response: T) -> Result<(), Error> {
		if self.selection_key.as_ref() != Some(selection_key) {
			// response from previous request => ignore
			return Ok(());
		}
		if !self.active_requests.remove(node) {
			return Err(Error::InvalidStateForRequest);
		}

		self.responses.insert(node.clone(), response);
		Ok(())
	}

	#[cfg(test)]
	/// Return job requests.
	pub fn job_requests(&self) -> Result<&BTreeSet<NodeId>, Error> {
		Ok(&self.active_requests)
	}

	/// Return job responses.
	pub fn job_responses(&self) -> Result<&BTreeMap<NodeId, T>, Error> {
		Ok(&self.responses)
	}

	/// Restart jobs.
	pub fn restart(&mut self) -> Result<(), Error> {
		self.selected_nodes.clear();
		self.active_requests.clear();
		self.responses.clear();
		Ok(())
	}

	/// When node is timeouted.
	pub fn node_timeouted(&mut self, node: &NodeId) -> Result<bool, Error> {
		self.core.node_timeouted(node)?;
		Ok(self.active_requests.remove(node) || self.responses.remove(node).is_some())
	}

	/// When session is timeouted.
	pub fn session_timeouted(&mut self) -> Result<(), Error> {
		for timeouted_node in &self.active_requests {
			self.core.node_timeouted(timeouted_node)?;
		}
		self.restart()
	}
}

#[cfg(test)]
mod tests {
	use key_server_cluster::Error;
	use super::Consensus;

	#[test]
	fn consensus_is_not_created_when_not_enough_nodes() {
		assert_eq!(Consensus::<i32>::new(0, vec![].into_iter().collect()).unwrap_err(), Error::InvalidThreshold);
	}
}
