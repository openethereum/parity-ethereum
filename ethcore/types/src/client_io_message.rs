
use ethereum_types::H256;
use bytes::Bytes;
use crate::snapshot_manifest::ManifestData;

/// Message type for external and internal events
#[derive(Debug)]
pub enum ClientIoMessage {
	/// Best Block Hash in chain has been changed
	NewChainHead,
	/// A block is ready
	BlockVerified,
	/// Begin snapshot restoration
	BeginRestoration(ManifestData),
	/// Feed a state chunk to the snapshot service
	FeedStateChunk(H256, Bytes),
	/// Feed a block chunk to the snapshot service
	FeedBlockChunk(H256, Bytes),
	/// Take a snapshot for the block with given number.
	TakeSnapshot(u64),
	// todo: hopefully we dont need this on in verifiers – gonna be tricky to wire up to ethcore
	///// Execute wrapped closure
	//Execute(Callback),
}

