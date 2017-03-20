use std::io::Cursor;
use std::u16;
use std::ops::Deref;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json;
use ethkey::{Public, Secret};
use ethcrypto::ecies;
use key_server_cluster::{Error, NodeId};
use key_server_cluster::message::{Message, ClusterMessage, EncryptionMessage, DecryptionMessage};

/// Size of serialized header.
pub const MESSAGE_HEADER_SIZE: usize = 4;

#[derive(Debug, PartialEq)]
/// Message header.
pub struct MessageHeader {
	/// Message/Header version.
	pub version: u8,
	/// Message kind.
	pub kind: u8,
	/// Message payload size (without header).
	pub size: u16,
}

#[derive(Debug, Clone, PartialEq)]
/// Serialized message.
pub struct SerializedMessage(Vec<u8>);

impl Deref for SerializedMessage {
	type Target = Vec<u8>;

	fn deref(&self) -> &Vec<u8> {
		&self.0
	}
}

impl Into<Vec<u8>> for SerializedMessage {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

pub fn serialize_message(message: Message) -> Result<SerializedMessage, Error> {
	let (message_kind, payload) = match message {
		Message::Cluster(ClusterMessage::NodePublicKey(payload))							=> (1, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::NodePrivateKeySignature(payload))					=> (2, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::KeepAlive(payload))								=> (3, serde_json::to_vec(&payload)),
		Message::Cluster(ClusterMessage::KeepAliveResponse(payload))						=> (4, serde_json::to_vec(&payload)),

		Message::Encryption(EncryptionMessage::InitializeSession(payload))					=> (50, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::ConfirmInitialization(payload))				=> (51, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::CompleteInitialization(payload))				=> (52, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::KeysDissemination(payload))					=> (53, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::Complaint(payload))							=> (54, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::ComplaintResponse(payload))					=> (55, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::PublicKeyShare(payload))						=> (56, serde_json::to_vec(&payload)),
		Message::Encryption(EncryptionMessage::SessionError(payload))						=> (57, serde_json::to_vec(&payload)),

		Message::Decryption(DecryptionMessage::InitializeDecryptionSession(payload))		=> (100, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::ConfirmDecryptionInitialization(payload))	=> (101, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::RequestPartialDecryption(payload))			=> (102, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::PartialDecryption(payload))					=> (103, serde_json::to_vec(&payload)),
		Message::Decryption(DecryptionMessage::DecryptionSessionError(payload))				=> (104, serde_json::to_vec(&payload)),
	};

	let payload = payload.map_err(|err| Error::Serde(format!("{}", err)))?;
	let payload_len = payload.len();
	if payload_len > u16::MAX as usize {
		return Err(Error::InvalidMessage);
	}

	let header = MessageHeader {
		kind: message_kind,
		version: 1,
		size: payload_len as u16,
	};

	let mut serialized_message = serialize_header(&header)?;
	serialized_message.extend(payload);
	Ok(SerializedMessage(serialized_message))
}

pub fn deserialize_message(header: &MessageHeader, payload: Vec<u8>) -> Result<Message, Error> {
	Ok(match header.kind {
		1	=> Message::Cluster(ClusterMessage::NodePublicKey(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		2	=> Message::Cluster(ClusterMessage::NodePrivateKeySignature(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		3	=> Message::Cluster(ClusterMessage::KeepAlive(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		4	=> Message::Cluster(ClusterMessage::KeepAliveResponse(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),

		50	=> Message::Encryption(EncryptionMessage::InitializeSession(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		51	=> Message::Encryption(EncryptionMessage::ConfirmInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		52	=> Message::Encryption(EncryptionMessage::CompleteInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		53	=> Message::Encryption(EncryptionMessage::KeysDissemination(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		54	=> Message::Encryption(EncryptionMessage::Complaint(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		55	=> Message::Encryption(EncryptionMessage::ComplaintResponse(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		56	=> Message::Encryption(EncryptionMessage::PublicKeyShare(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		57	=> Message::Encryption(EncryptionMessage::SessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),

		100	=> Message::Decryption(DecryptionMessage::InitializeDecryptionSession(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		101	=> Message::Decryption(DecryptionMessage::ConfirmDecryptionInitialization(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		102	=> Message::Decryption(DecryptionMessage::RequestPartialDecryption(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		103	=> Message::Decryption(DecryptionMessage::PartialDecryption(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),
		104	=> Message::Decryption(DecryptionMessage::DecryptionSessionError(serde_json::from_slice(&payload).map_err(|err| Error::Serde(format!("{}", err)))?)),

		_ => return Err(Error::Serde(format!("unknown message type {}", header.kind))),
	})
}

pub fn encrypt_message(key: &Public, message: SerializedMessage) -> Result<SerializedMessage, Error> {
// TODO
//	let mut encrypted_message = Vec::with_capacity(message.0.len());
//	encrypted_message.extend_from_slice(&message.0[0..MESSAGE_HEADER_SIZE]);
//	encrypted_message.extend(ecies::encrypt_single_message(key, &message.0[MESSAGE_HEADER_SIZE..])?);
//	Ok(SerializedMessage(encrypted_message))
	Ok(message)
}

pub fn decrypt_message(key: &Secret, payload: Vec<u8>) -> Result<Vec<u8>, Error> {
// TODO
//	Ok(ecies::decrypt_single_message(key, &payload)?)
	Ok(payload)
}

fn serialize_header(header: &MessageHeader) -> Result<Vec<u8>, Error> {
	let mut buffer = Vec::with_capacity(MESSAGE_HEADER_SIZE);
	buffer.write_u8(header.version)?;
	buffer.write_u8(header.kind)?;
	buffer.write_u16::<LittleEndian>(header.size)?;
	Ok(buffer)
}

pub fn deserialize_header(data: Vec<u8>) -> Result<MessageHeader, Error> {
	let mut reader = Cursor::new(data);
	Ok(MessageHeader {
		version: reader.read_u8()?,
		kind: reader.read_u8()?,
		size: reader.read_u16::<LittleEndian>()?,
	})
}

#[cfg(test)]
pub mod tests {
	use std::io;
	use ethcrypto::ecdh::agree;
	use ethkey::{Random, Generator, KeyPair, Public};
	use key_server_cluster::message::Message;
	use super::{MESSAGE_HEADER_SIZE, MessageHeader, SerializedMessage, serialize_message, deserialize_message,
		encrypt_message, decrypt_message, serialize_header, deserialize_header};

	pub struct TestIo {
		self_key_pair: KeyPair,
		peer_public: Public,
		input_buffer: io::Cursor<Vec<u8>>,
		output_buffer: Vec<u8>,
		expected_output_buffer: Vec<u8>,
	}

	impl TestIo {
		pub fn new(self_key_pair: KeyPair, peer_public: Public) -> Self {
			TestIo {
				self_key_pair: self_key_pair,
				peer_public: peer_public,
				input_buffer: io::Cursor::new(Vec::new()),
				output_buffer: Vec::new(),
				expected_output_buffer: Vec::new(),
			}
		}

		pub fn self_key_pair(&self) -> &KeyPair {
			&self.self_key_pair
		}

		pub fn peer_public(&self) -> &Public {
			&self.peer_public
		}

		pub fn add_input_message(&mut self, message: Message) {
			let serialized_message = serialize_message(message).unwrap();
			let serialized_message: Vec<_> = serialized_message.into();
			let input_buffer = self.input_buffer.get_mut();
			for b in serialized_message {
				input_buffer.push(b);
			}
		}

		pub fn add_output_message(&mut self, message: Message) {
			let serialized_message = serialize_message(message).unwrap();
			let serialized_message: Vec<_> = serialized_message.into();
			self.expected_output_buffer.extend(serialized_message);
		}

		pub fn assert_output(&self) {
			assert_eq!(self.output_buffer, self.expected_output_buffer);
		}
	}

	impl io::Read for TestIo {
		fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
			io::Read::read(&mut self.input_buffer, buf)
		}
	}

	impl io::Write for TestIo {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			io::Write::write(&mut self.output_buffer, buf)
		}

		fn flush(&mut self) -> io::Result<()> {
			io::Write::flush(&mut self.output_buffer)
		}
	}

	#[test]
	fn header_serialization_works() {
		let header = MessageHeader {
			kind: 1,
			version: 2,
			size: 3,
		};

		let serialized_header = serialize_header(&header).unwrap();
		assert_eq!(serialized_header.len(), MESSAGE_HEADER_SIZE);

		let deserialized_header = deserialize_header(serialized_header).unwrap();
		assert_eq!(deserialized_header, header);
	}

	// TODO: #[test]
	fn message_encryption_works() {
		let key_pair1 = Random.generate().unwrap();
		let key_pair2 = Random.generate().unwrap();
		let secret1 = agree(key_pair1.secret(), key_pair2.public()).unwrap();
		let secret2 = agree(key_pair2.secret(), key_pair1.public()).unwrap();
		assert_eq!(secret1, secret2);

		let plain_message = SerializedMessage(vec![1, 2, 3, 4]);
		let shared_key_pair = KeyPair::from_secret(secret1).unwrap();

		let encrypted_message = encrypt_message(shared_key_pair.public(), plain_message.clone()).unwrap();
		assert!(plain_message != encrypted_message);

		let decrypted_message = decrypt_message(shared_key_pair.secret(), encrypted_message.0).unwrap();
		assert_eq!(decrypted_message, plain_message.0);
	}
}
