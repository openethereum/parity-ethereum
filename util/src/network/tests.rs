use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::thread;
use std::time::*;
use common::*;
use network::*;
use io::TimerToken;
use crypto::KeyPair;

pub struct TestProtocol {
	pub packet: Mutex<Bytes>,
	pub got_timeout: AtomicBool,
}

impl Default for TestProtocol {
	fn default() -> Self {
		TestProtocol { 
			packet: Mutex::new(Vec::new()), 
			got_timeout: AtomicBool::new(false), 
		}
	}
}

#[derive(Clone)]
pub struct TestProtocolMessage {
	payload: u32,
}

impl TestProtocol {
	/// Creates and register protocol with the network service
	pub fn register(service: &mut NetworkService<TestProtocolMessage>) -> Arc<TestProtocol> {
		let handler = Arc::new(TestProtocol::default());
		service.register_protocol(handler.clone(), "test", &[42u8, 43u8]).expect("Error registering test protocol handler");
		handler
	}

	pub fn got_packet(&self) -> bool {
		self.packet.lock().unwrap().deref()[..] == b"hello"[..]
	}

	pub fn got_timeout(&self) -> bool {
		self.got_timeout.load(AtomicOrdering::Relaxed)
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

	fn connected(&self, io: &NetworkContext<TestProtocolMessage>, _peer: &PeerId) {
		io.respond(33, "hello".to_owned().into_bytes()).unwrap();
	}

	fn disconnected(&self, _io: &NetworkContext<TestProtocolMessage>, _peer: &PeerId) {
	}

	/// Timer function called after a timeout created with `NetworkContext::timeout`.
	fn timeout(&self, _io: &NetworkContext<TestProtocolMessage>, timer: TimerToken) {
		assert_eq!(timer, 0);
		self.got_timeout.store(true, AtomicOrdering::Relaxed);
	}
}


#[test]
fn net_service() {
	let mut service = NetworkService::<TestProtocolMessage>::start(NetworkConfiguration::new()).expect("Error creating network service");
	service.register_protocol(Arc::new(TestProtocol::default()), "myproto", &[1u8]).unwrap();
}

#[test]
fn net_connect() {
	let key1 = KeyPair::create().unwrap();
	let mut config1 = NetworkConfiguration::new_with_port(30344);
	config1.use_secret = Some(key1.secret().clone());
	config1.boot_nodes = vec![ ];
	let mut config2 = NetworkConfiguration::new_with_port(30345);
	config2.boot_nodes = vec![ format!("enode://{}@127.0.0.1:30344", key1.public().hex()) ];
	let mut service1 = NetworkService::<TestProtocolMessage>::start(config1).unwrap();
	let mut service2 = NetworkService::<TestProtocolMessage>::start(config2).unwrap();
	let handler1 = TestProtocol::register(&mut service1);
	let handler2 = TestProtocol::register(&mut service2);
	while !handler1.got_packet() && !handler2.got_packet() {
		thread::sleep(Duration::from_millis(50));
	}
	assert!(service1.stats().sessions() >= 1);
	assert!(service2.stats().sessions() >= 1);
}

#[test]
fn net_timeout() {
	let config = NetworkConfiguration::new_with_port(30346);
	let mut service = NetworkService::<TestProtocolMessage>::start(config).unwrap();
	let handler = TestProtocol::register(&mut service);
	while !handler.got_timeout() {
		thread::sleep(Duration::from_millis(50));
	}
}
