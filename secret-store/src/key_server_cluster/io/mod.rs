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

mod deadline;
mod handshake;
mod message;
mod read_header;
mod read_message;
mod read_payload;
mod shared_tcp_stream;
mod write_message;

pub use self::{
    deadline::{deadline, Deadline, DeadlineStatus},
    handshake::{accept_handshake, handshake, Handshake, HandshakeResult},
    message::{
        deserialize_message, encrypt_message, fix_shared_key, serialize_message, MessageHeader,
        SerializedMessage,
    },
    read_header::{read_header, ReadHeader},
    read_message::{read_encrypted_message, read_message, ReadMessage},
    read_payload::{read_encrypted_payload, read_payload, ReadPayload},
    shared_tcp_stream::SharedTcpStream,
    write_message::{write_encrypted_message, write_message, WriteMessage},
};
