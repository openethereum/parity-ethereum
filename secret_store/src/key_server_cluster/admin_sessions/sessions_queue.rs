use std::sync::Arc;
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, KeyStorage, DummyAclStorage, DocumentKeyShare};

pub struct SessionQueue {
//	known_sessions: Box<Iterator<Item=(ServerKeyId, DocumentKeyShare)> + 'a>
}