// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use super::types::ServerKeyId;

#[cfg(test)]
pub use self::cluster::tests::DummyClusterClient;
pub use self::{
    cluster::{new_network_cluster, ClusterClient, ClusterConfiguration, ClusterCore},
    cluster_connections_net::NetConnectionsManagerConfig,
    cluster_sessions::{ClusterSession, ClusterSessionsListener},
};
pub use super::{
    acl_storage::AclStorage,
    key_server_set::{
        is_migration_required, KeyServerSet, KeyServerSetMigration, KeyServerSetSnapshot,
    },
    key_storage::{DocumentKeyShare, DocumentKeyShareVersion, KeyStorage},
    serialization::{
        SerializableAddress, SerializableH256, SerializableMessageHash, SerializablePublic,
        SerializableRequester, SerializableSecret, SerializableSignature,
    },
    traits::NodeKeyPair,
    types::{EncryptedDocumentKeyShadow, Error, NodeId, Requester},
};

pub use super::acl_storage::DummyAclStorage;
#[cfg(test)]
pub use super::key_server_set::tests::MapKeyServerSet;
#[cfg(test)]
pub use super::key_storage::tests::DummyKeyStorage;
#[cfg(test)]
pub use super::node_key_pair::PlainNodeKeyPair;

pub type SessionId = ServerKeyId;

/// Session metadata.
#[derive(Debug, Clone)]
pub struct SessionMeta {
    /// Key id.
    pub id: SessionId,
    /// Id of node, which has started this session.
    pub master_node_id: NodeId,
    /// Id of node, on which this session is running.
    pub self_node_id: NodeId,
    /// Session threshold.
    pub threshold: usize,
    /// Count of all configured key server nodes (valid at session start time).
    pub configured_nodes_count: usize,
    /// Count of all connected key server nodes (valid at session start time).
    pub connected_nodes_count: usize,
}

mod admin_sessions;
mod client_sessions;

pub use self::admin_sessions::{
    key_version_negotiation_session, servers_set_change_session, share_add_session,
    share_change_session,
};

pub use self::client_sessions::{
    decryption_session, encryption_session, generation_session, signing_session_ecdsa,
    signing_session_schnorr,
};

mod cluster;
mod cluster_connections;
mod cluster_connections_net;
mod cluster_message_processor;
mod cluster_sessions;
mod cluster_sessions_creator;
mod connection_trigger;
mod connection_trigger_with_migration;
mod io;
mod jobs;
pub mod math;
mod message;
mod net;
