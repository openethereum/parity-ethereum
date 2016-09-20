use ethcore_signer::ServerBuilder;
use ethcore_signer::Server;
use rpc::ConfirmationsQueue;
use std::sync::Arc;
use std::time::{Duration};
use std::thread;
use rand;
use tempdir::TempDir;
use std::path::PathBuf;
use std::fs::{File, create_dir_all};
use std::io::Write;

// mock server
pub fn serve() -> (Server, usize, TempDir, Arc<ConfirmationsQueue>) {
	let queue = Arc::new(ConfirmationsQueue::default());
	let dir = TempDir::new("auth").unwrap();

	let mut authpath = PathBuf::from(dir.path());
	create_dir_all(&authpath).unwrap();
	authpath.push("authcodes");
	let mut authfile = File::create(&authpath).unwrap();
	authfile.write_all(b"zzzRo0IzGi04mzzz\n").unwrap();

	let builder = ServerBuilder::new(queue.clone(), authpath);
	let port = 35000 + rand::random::<usize>() % 10000;
	let res = builder.start(format!("127.0.0.1:{}", port).parse().unwrap()).unwrap();

	thread::sleep(Duration::from_millis(25));
	(res, port, dir, queue)
}
