// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::thread;
use std::time::*;
use common::*;
use network::*;
use io::TimerToken;
use crypto::KeyPair;

pub struct TestProtocol {
	drop_session: bool,
	pub packet: Mutex<Bytes>,
	pub got_timeout: AtomicBool,
	pub got_disconnect: AtomicBool,
}

#[derive(Clone)]
pub struct TestProtocolMessage {
	payload: u32,
}

impl TestProtocol {
	pub fn new(drop_session: bool) -> Self {
		TestProtocol { 
			packet: Mutex::new(Vec::new()), 
			got_timeout: AtomicBool::new(false), 
			got_disconnect: AtomicBool::new(false), 
			drop_session: drop_session,
		}
	}
	/// Creates and register protocol with the network service
	pub fn register(service: &mut NetworkService<TestProtocolMessage>, drop_session: bool) -> Arc<TestProtocol> {
		let handler = Arc::new(TestProtocol::new(drop_session));
		service.register_protocol(handler.clone(), "test", &[42u8, 43u8]).expect("Error registering test protocol handler");
		handler
	}

	pub fn got_packet(&self) -> bool {
		self.packet.lock().unwrap().deref()[..] == b"hello"[..]
	}

	pub fn got_timeout(&self) -> bool {
		self.got_timeout.load(AtomicOrdering::Relaxed)
	}

	pub fn got_disconnect(&self) -> bool {
		self.got_disconnect.load(AtomicOrdering::Relaxed)
	}
}

impl NetworkProtocolHandler<TestProtocolMessage> for TestProtocol {
	fn initialize(&self, io: &NetworkContext<TestProtocolMessage>) {
		io.register_timer(0, 10).unwrap();
	}

	fn read(&self, _io: &NetworkContext<TestProtocolMessage>, _peer: &PeerId, packet_id: u8, data: &[u8]) {
		assert_eq!(packet_id, 33);
		self.packet.lock().unwrap().extend(data);
	}

	fn connected(&self, io: &NetworkContext<TestProtocolMessage>, peer: &PeerId) {
		assert!(io.peer_info(*peer).contains("Parity"));
		if self.drop_session {
			io.disconnect_peer(*peer)
		} else {
			io.respond(33, "hello".to_owned().into_bytes()).unwrap();
		}
	}

	fn disconnected(&self, _io: &NetworkContext<TestProtocolMessage>, _peer: &PeerId) {
		self.got_disconnect.store(true, AtomicOrdering::Relaxed);
	}

	/// Timer function called after a timeout created with `NetworkContext::timeout`.
	fn timeout(&self, io: &NetworkContext<TestProtocolMessage>, timer: TimerToken) {
		io.message(TestProtocolMessage { payload: 22 });
		assert_eq!(timer, 0);
		self.got_timeout.store(true, AtomicOrdering::Relaxed);
	}
}


#[test]
fn net_service() {
	let mut service = NetworkService::<TestProtocolMessage>::start(NetworkConfiguration::new_local()).expect("Error creating network service");
	service.register_protocol(Arc::new(TestProtocol::new(false)), "myproto", &[1u8]).unwrap();
}

#[test]
fn net_connect() {
	::log::init_log();
	let key1 = KeyPair::create().unwrap();
	let mut config1 = NetworkConfiguration::new_local();
	config1.use_secret = Some(key1.secret().clone());
	config1.boot_nodes = vec![ ];
	let mut service1 = NetworkService::<TestProtocolMessage>::start(config1).unwrap();
	let handler1 = TestProtocol::register(&mut service1, false);
	let mut config2 = NetworkConfiguration::new_local();
	info!("net_connect: local URL: {}", service1.local_url());
	config2.boot_nodes = vec![ service1.local_url() ];
	let mut service2 = NetworkService::<TestProtocolMessage>::start(config2).unwrap();
	let handler2 = TestProtocol::register(&mut service2, false);
	while !handler1.got_packet() && !handler2.got_packet() && (service1.stats().sessions() == 0 || service2.stats().sessions() == 0) {
		thread::sleep(Duration::from_millis(50));
	}
	assert!(service1.stats().sessions() >= 1);
	assert!(service2.stats().sessions() >= 1);
}

#[test]
fn net_disconnect() {
	let key1 = KeyPair::create().unwrap();
	let mut config1 = NetworkConfiguration::new_local();
	config1.use_secret = Some(key1.secret().clone());
	config1.boot_nodes = vec![ ];
	let mut service1 = NetworkService::<TestProtocolMessage>::start(config1).unwrap();
	let handler1 = TestProtocol::register(&mut service1, false);
	let mut config2 = NetworkConfiguration::new_local();
	config2.boot_nodes = vec![ service1.local_url() ];
	let mut service2 = NetworkService::<TestProtocolMessage>::start(config2).unwrap();
	let handler2 = TestProtocol::register(&mut service2, true);
	while !(handler1.got_disconnect() && handler2.got_disconnect()) {
		thread::sleep(Duration::from_millis(50));
	}
	assert!(handler1.got_disconnect());
	assert!(handler2.got_disconnect());
}

#[test]
fn net_timeout() {
	let config = NetworkConfiguration::new_local();
	let mut service = NetworkService::<TestProtocolMessage>::start(config).unwrap();
	let handler = TestProtocol::register(&mut service, false);
	while !handler.got_timeout() {
		thread::sleep(Duration::from_millis(50));
	}
}
