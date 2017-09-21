use std::collections::{BTreeMap, BTreeSet};
use key_server_cluster::{Error, NodeId};
use key_server_cluster::jobs::job_session::{JobExecutor, JobTransport, JobPartialRequestAction, JobPartialResponseAction};

pub struct DummyJob;

impl JobExecutor for DummyJob {
	type PartialJobRequest = ();
	type PartialJobResponse = ();
	type JobResponse = ();

	fn prepare_partial_request(&self, _n: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<(), Error> { unreachable!() }
	fn process_partial_request(&mut self, _r: ()) -> Result<JobPartialRequestAction<()>, Error> { unreachable!() }
	fn check_partial_response(&self, _r: &()) -> Result<JobPartialResponseAction, Error> { unreachable!() }
	fn compute_response(&self, _r: &BTreeMap<NodeId, ()>) -> Result<(), Error> { unreachable!() }
}

pub struct DummyJobTransport;

impl JobTransport for DummyJobTransport {
	type PartialJobRequest = ();
	type PartialJobResponse = ();

	fn send_partial_request(&self, _node: &NodeId, request: ()) -> Result<(), Error> { unreachable!() }
	fn send_partial_response(&self, _node: &NodeId, response: ()) -> Result<(), Error> { unreachable!() }
}
