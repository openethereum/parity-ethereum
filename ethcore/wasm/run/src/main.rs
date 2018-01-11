extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate ethereum_types;
extern crate ethjson;
extern crate wasm;
extern crate vm;
extern crate clap;
extern crate ethcore_logger;
extern crate rustc_hex;

mod fixture;
mod runner;

use fixture::Fixture;
use clap::{App, Arg};
use std::fs;

fn main() {
	::ethcore_logger::init_log();

	let matches = App::new("pwasm-run-test")
		.arg(Arg::with_name("target")
			.index(1)
			.required(true)
			.multiple(true)
			.help("JSON fixture"))
		.get_matches();

	let mut exit_code = 0;

	for target in matches.values_of("target").expect("No target parameter") {
		let mut f = fs::File::open(target).expect("Failed to open file");
		let fixtures: Vec<Fixture> = serde_json::from_reader(&mut f).expect("Failed to deserialize json");

		for fixture in fixtures.into_iter() {
			let fails = runner::run_fixture(&fixture);
			for fail in fails.iter() {
				exit_code = 1;
				println!("Failed assert in test \"{}\" ('{}'): {}", fixture.caption.as_ref(), target, fail);
			}
		}
	}

	std::process::exit(exit_code);
}
