extern crate https_fetch;

use std::io;
use https_fetch::*;

fn main() {
	let client = Client::new().unwrap();

	client.fetch(Url::new("github.com", 443, "/").unwrap(), Box::new(io::stdout()), |result| {
		assert!(result.is_ok());
	}).unwrap();
}
