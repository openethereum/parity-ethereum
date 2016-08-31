extern crate https_fetch;

use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use https_fetch::*;

fn main() {
	let client = Client::new().unwrap();
	let aborted = Arc::new(AtomicBool::new(false));

	client.fetch(Url::new("github.com", 443, "/").unwrap(), Box::new(io::stdout()), aborted, |result| {
		assert!(result.is_ok());
	}).unwrap();
}
