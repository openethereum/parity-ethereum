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

use std::collections::{BTreeMap, BTreeSet};
use key_server_cluster::{Error, NodeId};
use key_server_cluster::jobs::job_session::{JobExecutor, JobTransport, JobPartialRequestAction, JobPartialResponseAction};

/// No-work job to use in generics (TODO: create separate ShareChangeConsensusSession && remove this)
pub struct DummyJob;

impl JobExecutor for DummyJob {
	type PartialJobRequest = ();
	type PartialJobResponse = ();
	type JobResponse = ();

	fn prepare_partial_request(&self, _n: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<(), Error> {
		unreachable!("dummy job methods are never called")
	}

	fn process_partial_request(&mut self, _r: ()) -> Result<JobPartialRequestAction<()>, Error> {
		unreachable!("dummy job methods are never called")
	}

	fn check_partial_response(&self, _r: &()) -> Result<JobPartialResponseAction, Error> {
		unreachable!("dummy job methods are never called")
	}

	fn compute_response(&self, _r: &BTreeMap<NodeId, ()>) -> Result<(), Error> {
		unreachable!("dummy job methods are never called")
	}
}

/// No-work job transport to use in generics (TODO: create separate ShareChangeConsensusSession && remove this)
pub struct DummyJobTransport;

impl JobTransport for DummyJobTransport {
	type PartialJobRequest = ();
	type PartialJobResponse = ();

	fn send_partial_request(&self, _node: &NodeId, _request: ()) -> Result<(), Error> {
		unreachable!("dummy transport methods are never called")
	}

	fn send_partial_response(&self, _node: &NodeId, _response: ()) -> Result<(), Error> {
		unreachable!("dummy transport methods are never called")
	}
}
