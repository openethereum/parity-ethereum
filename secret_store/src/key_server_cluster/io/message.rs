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

use std::io::Cursor;
use std::u16;
use std::ops::Deref;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json;
use ethcrypto::ecies::{encrypt_single_message, decrypt_single_message};
use ethkey::{Secret, KeyPair};
use ethkey::math::curve_order;
use ethereum_types::{H256, U256};
use key_server_cluster::Error;
use key_server_cluster::message::{Message, ClusterMessage, GenerationMessage, EncryptionMessage,
	DecryptionMessage, SigningMessage, ServersSetChangeMessage, ShareAddMessage, KeyVersionNegotiationMessage};

/// Size of serialized header.
pub const MESSAGE_HEADER_SIZE: usize = 18;
/// Current header version.
pub const CURRENT_HEADER_VERSION: u64 = 1;

/// Message header.
#[derive(Debug, PartialEq)]
pub struct MessageHeader {
	/// Message/Header version.
	pub version: u64,
	/// Message kind.
	pub kind: u64,
	/// Message payload size (without header).
	pub size: u16,
}

/// Serialized message.
#[derive(Debug, Clone, PartialEq)]
pub struct SerializedMessage(Vec<u8>);

impl Deref for SerializedMessage {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		&self.0
	}
}

impl Into<Vec<u8>> for SerializedMessage {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

/// Serialize message.
pub fn serialize_message(message: Message) -> Result<SerializedMessage, Error> {
	let (message_kind, payload) = match message {
		Message::Cluster(ClusterMessage::NodePublicKey(payload))							=> (1, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::NodePrivateKeySignature(payload))					=> (2, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::KeepAlive(payload))								=> (3, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::KeepAliveResponse(payload))						=> (4, serde_json::to_vec(&payload)),

		Message::Generation(GenerationMessage::InitializeSession(payload))					=> (50, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::ConfirmInitialization(payload))				=> (51, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::CompleteInitialization(payload))				=> (52, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::KeysDissemination(payload))					=> (53, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::PublicKeyShare(payload))						=> (54, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::SessionError(payload))						=> (55, serde_json::to_vec(&payload)),
		Message::Generation(GenerationMessage::SessionCompleted(payload))					=> (56, serde_json::to_vec(&payload)),

		Message::Encryption(EncryptionMessage::InitializeEncryptionSession(payload))		=> (100, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::ConfirmEncryptionInitialization(payload))	=> (101, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::EncryptionSessionError(payload))				=> (102, serde_json::to_vec(&payload)),

		Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(payload))			=> (150, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::RequestPartialDecryption(payload))			=> (151, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::PartialDecryption(payload))					=> (152, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::DecryptionSessionError(payload))				=> (153, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::DecryptionSessionCompleted(payload))			=> (154, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::DecryptionSessionDelegation(payload))		=> (155, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::DecryptionSessionDelegationCompleted(payload))
																							=> (156, serde_json::to_vec(&payload)),

		Message::Signing(SigningMessage::SigningConsensusMessage(payload))					=> (200, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::SigningGenerationMessage(payload))					=> (201, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::RequestPartialSignature(payload))					=> (202, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::PartialSignature(payload))							=> (203, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::SigningSessionError(payload))						=> (204, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::SigningSessionCompleted(payload))					=> (205, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::SigningSessionDelegation(payload))					=> (206, serde_json::to_vec(&payload)),
		Message::Signing(SigningMessage::SigningSessionDelegationCompleted(payload))		=> (207, serde_json::to_vec(&payload)),

		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(payload))
																							=> (250, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::UnknownSessionsRequest(payload)) => (251, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::UnknownSessions(payload))		=> (252, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(payload))
																							=> (253, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::InitializeShareChangeSession(payload))
																							=> (254, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(payload))
																							=> (255, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegate(payload))
																							=> (256, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegateResponse(payload))
																							=> (257, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareAddMessage(payload))
																							=> (258, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeError(payload))	=> (261, serde_json::to_vec(&payload)),
		Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeCompleted(payload))
																							=> (262, serde_json::to_vec(&payload)),

		Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(payload))				=> (300, serde_json::to_vec(&payload)),
		Message::ShareAdd(ShareAddMessage::KeyShareCommon(payload))							=> (301, serde_json::to_vec(&payload)),
		Message::ShareAdd(ShareAddMessage::NewKeysDissemination(payload))					=> (302, serde_json::to_vec(&payload)),
		Message::ShareAdd(ShareAddMessage::ShareAddError(payload))							=> (303, serde_json::to_vec(&payload)),

		Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::RequestKeyVersions(payload))
																							=> (450, serde_json::to_vec(&payload)),
		Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::KeyVersions(payload))
																							=> (451, serde_json::to_vec(&payload)),
		Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::KeyVersionsError(payload))
																							=> (452, serde_json::to_vec(&payload)),
	};

	let payload = payload.map_err(|err| Error::Serde(err.to_string()))?;
	build_serialized_message(MessageHeader {
		kind: message_kind,
		version: CURRENT_HEADER_VERSION,
		size: 0,
	}, payload)
}

/// Deserialize message.
pub fn deserialize_message(header: &MessageHeader, payload: Vec<u8>) -> Result<Message, Error> {
	Ok(match header.kind {
		1	=> Message::Cluster(ClusterMessage::NodePublicKey(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		2	=> Message::Cluster(ClusterMessage::NodePrivateKeySignature(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		3	=> Message::Cluster(ClusterMessage::KeepAlive(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		4	=> Message::Cluster(ClusterMessage::KeepAliveResponse(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		50	=> Message::Generation(GenerationMessage::InitializeSession(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		51	=> Message::Generation(GenerationMessage::ConfirmInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		52	=> Message::Generation(GenerationMessage::CompleteInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		53	=> Message::Generation(GenerationMessage::KeysDissemination(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		54	=> Message::Generation(GenerationMessage::PublicKeyShare(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		55	=> Message::Generation(GenerationMessage::SessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		56	=> Message::Generation(GenerationMessage::SessionCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		100	=> Message::Encryption(EncryptionMessage::InitializeEncryptionSession(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		101	=> Message::Encryption(EncryptionMessage::ConfirmEncryptionInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		102	=> Message::Encryption(EncryptionMessage::EncryptionSessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		150	=> Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		151	=> Message::Decryption(DecryptionMessage::RequestPartialDecryption(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		152	=> Message::Decryption(DecryptionMessage::PartialDecryption(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		153	=> Message::Decryption(DecryptionMessage::DecryptionSessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		154	=> Message::Decryption(DecryptionMessage::DecryptionSessionCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		155	=> Message::Decryption(DecryptionMessage::DecryptionSessionDelegation(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		156	=> Message::Decryption(DecryptionMessage::DecryptionSessionDelegationCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		200	=> Message::Signing(SigningMessage::SigningConsensusMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		201	=> Message::Signing(SigningMessage::SigningGenerationMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		202	=> Message::Signing(SigningMessage::RequestPartialSignature(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		203	=> Message::Signing(SigningMessage::PartialSignature(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		204	=> Message::Signing(SigningMessage::SigningSessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		205	=> Message::Signing(SigningMessage::SigningSessionCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		206	=> Message::Signing(SigningMessage::SigningSessionDelegation(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		207	=> Message::Signing(SigningMessage::SigningSessionDelegationCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		250	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		251	=> Message::ServersSetChange(ServersSetChangeMessage::UnknownSessionsRequest(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		252	=> Message::ServersSetChange(ServersSetChangeMessage::UnknownSessions(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		253 => Message::ServersSetChange(ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		254 => Message::ServersSetChange(ServersSetChangeMessage::InitializeShareChangeSession(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		255 => Message::ServersSetChange(ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		256	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegate(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		257	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegateResponse(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		258	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareAddMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		261	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		262	=> Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeCompleted(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		300 => Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		301 => Message::ShareAdd(ShareAddMessage::KeyShareCommon(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		302 => Message::ShareAdd(ShareAddMessage::NewKeysDissemination(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		303 => Message::ShareAdd(ShareAddMessage::ShareAddError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		450 => Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::RequestKeyVersions(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		451 => Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::KeyVersions(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),
		452 => Message::KeyVersionNegotiation(KeyVersionNegotiationMessage::KeyVersionsError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(err.to_string()))?)),

		_ => return Err(Error::Serde(format!("unknown message type {}", header.kind))),
	})
}

/// Encrypt serialized message.
pub fn encrypt_message(key: &KeyPair, message: SerializedMessage) -> Result<SerializedMessage, Error> {
	let mut header: Vec<_> = message.into();
	let payload = header.split_off(MESSAGE_HEADER_SIZE);
	let encrypted_payload = encrypt_single_message(key.public(), &payload)?;

	let header = deserialize_header(&header)?;
	build_serialized_message(header, encrypted_payload)
}

/// Decrypt serialized message.
pub fn decrypt_message(key: &KeyPair, payload: Vec<u8>) -> Result<Vec<u8>, Error> {
	Ok(decrypt_single_message(key.secret(), &payload)?)
}

/// Fix shared encryption key.
pub fn fix_shared_key(shared_secret: &Secret) -> Result<KeyPair, Error> {
	// secret key created in agree function is invalid, as it is not calculated mod EC.field.n
	// => let's do it manually
	let shared_secret: H256 = (**shared_secret).into();
	let shared_secret: U256 = shared_secret.into();
	let shared_secret: H256 = (shared_secret % curve_order()).into();
	let shared_key_pair = KeyPair::from_secret_slice(&*shared_secret)?;
	Ok(shared_key_pair)
}

/// Serialize message header.
fn serialize_header(header: &MessageHeader) -> Result<Vec<u8>, Error> {
	let mut buffer = Vec::with_capacity(MESSAGE_HEADER_SIZE);
	buffer.write_u64::<LittleEndian>(header.version)?;
	buffer.write_u64::<LittleEndian>(header.kind)?;
	buffer.write_u16::<LittleEndian>(header.size)?;
	Ok(buffer)
}

/// Deserialize message header.
pub fn deserialize_header(data: &[u8]) -> Result<MessageHeader, Error> {
	let mut reader = Cursor::new(data);
	let version = reader.read_u64::<LittleEndian>()?;
	if version != CURRENT_HEADER_VERSION {
		return Err(Error::InvalidMessageVersion);
	}

	Ok(MessageHeader {
		version: version,
		kind: reader.read_u64::<LittleEndian>()?,
		size: reader.read_u16::<LittleEndian>()?,
	})
}

/// Build serialized message from header && payload
fn build_serialized_message(mut header: MessageHeader, payload: Vec<u8>) -> Result<SerializedMessage, Error> {
	let payload_len = payload.len();
	if payload_len > u16::MAX as usize {
		return Err(Error::InvalidMessage);
	}
	header.size = payload.len() as u16;

	let mut message = serialize_header(&header)?;
	message.extend(payload);
	Ok(SerializedMessage(message))
}

#[cfg(test)]
pub mod tests {
	use std::io;
	use futures::Poll;
	use tokio_io::{AsyncRead, AsyncWrite};
	use ethkey::{Random, Generator, KeyPair};
	use ethcrypto::ecdh::agree;
	use key_server_cluster::Error;
	use key_server_cluster::message::Message;
	use super::{MESSAGE_HEADER_SIZE, CURRENT_HEADER_VERSION, MessageHeader, fix_shared_key, encrypt_message,
		serialize_message, serialize_header, deserialize_header};

	pub struct TestIo {
		self_key_pair: KeyPair,
		self_session_key_pair: KeyPair,
		peer_key_pair: KeyPair,
		peer_session_key_pair: KeyPair,
		shared_key_pair: KeyPair,
		input_buffer: io::Cursor<Vec<u8>>,
	}

	impl TestIo {
		pub fn new() -> Self {
			let self_session_key_pair = Random.generate().unwrap();
			let peer_session_key_pair = Random.generate().unwrap();
			let self_key_pair = Random.generate().unwrap();
			let peer_key_pair = Random.generate().unwrap();
			let shared_key_pair = fix_shared_key(&agree(self_session_key_pair.secret(), peer_session_key_pair.public()).unwrap()).unwrap();
			TestIo {
				self_key_pair: self_key_pair,
				self_session_key_pair: self_session_key_pair,
				peer_key_pair: peer_key_pair,
				peer_session_key_pair: peer_session_key_pair,
				shared_key_pair: shared_key_pair,
				input_buffer: io::Cursor::new(Vec::new()),
			}
		}

		pub fn self_key_pair(&self) -> &KeyPair {
			&self.self_key_pair
		}

		pub fn self_session_key_pair(&self) -> &KeyPair {
			&self.self_session_key_pair
		}

		pub fn peer_key_pair(&self) -> &KeyPair {
			&self.peer_key_pair
		}

		pub fn peer_session_key_pair(&self) -> &KeyPair {
			&self.peer_session_key_pair
		}

		pub fn shared_key_pair(&self) -> &KeyPair {
			&self.shared_key_pair
		}

		pub fn add_input_message(&mut self, message: Message) {
			let serialized_message = serialize_message(message).unwrap();
			let serialized_message: Vec<_> = serialized_message.into();
			let input_buffer = self.input_buffer.get_mut();
			for b in serialized_message {
				input_buffer.push(b);
			}
		}

		pub fn add_encrypted_input_message(&mut self, message: Message) {
			let serialized_message = encrypt_message(&self.shared_key_pair, serialize_message(message).unwrap()).unwrap();
			let serialized_message: Vec<_> = serialized_message.into();
			let input_buffer = self.input_buffer.get_mut();
			for b in serialized_message {
				input_buffer.push(b);
			}
		}
	}

	impl AsyncRead for TestIo {}

	impl AsyncWrite for TestIo {
		fn shutdown(&mut self) -> Poll<(), io::Error> {
			Ok(().into())
		}
	}

	impl io::Read for TestIo {
		fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
			io::Read::read(&mut self.input_buffer, buf)
		}
	}

	impl io::Write for TestIo {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			Ok(buf.len())
		}

		fn flush(&mut self) -> io::Result<()> {
			Ok(())
		}
	}

	#[test]
	fn header_serialization_works() {
		let header = MessageHeader {
			kind: 1,
			version: CURRENT_HEADER_VERSION,
			size: 3,
		};

		let serialized_header = serialize_header(&header).unwrap();
		assert_eq!(serialized_header.len(), MESSAGE_HEADER_SIZE);

		let deserialized_header = deserialize_header(&serialized_header).unwrap();
		assert_eq!(deserialized_header, header);
	}

	#[test]
	fn deserializing_header_of_wrong_version_fails() {
		let header = MessageHeader {
			kind: 1,
			version: CURRENT_HEADER_VERSION + 1,
			size: 3,
		};

		assert_eq!(deserialize_header(&serialize_header(&header).unwrap()).unwrap_err(), Error::InvalidMessageVersion);
	}
}
