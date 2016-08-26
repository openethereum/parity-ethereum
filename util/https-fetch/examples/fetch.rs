extern crate https_fetch;

use std::io;
use https_fetch::*;

fn main() {
	let client = Client::new().unwrap();

	let rx = client.fetch(Url::new("github.com", 443, "/").unwrap(), Box::new(io::stdout())).unwrap();

	let result = rx.recv().unwrap();

	assert!(result.is_ok());
}
