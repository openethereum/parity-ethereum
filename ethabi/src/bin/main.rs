extern crate docopt;
extern crate rustc_serialize;
extern crate ethabi;

use docopt::Docopt;
use std::process;

pub const ETHABI: &'static str = r#"
Ethereum ABI coder.
  Copyright 2016 Ethcore (UK) Limited

Usage:
    ethabi encode abi <abi-path> <function-name> [<param>]... [-l | --lenient]
    ethabi encode params [-p <type> <param>]... [-l | --lenient]
    ethabi decode abi <abi-path> <function-name> <data>
    ethabi decode params [-p <type>]... <data>
    ethabi [--help]

Options:
    -h, --help         Display this message and exit.
    -l, --lenient      Allow short representation of input params.

Commands:
    encode             Encode ABI call.
    decode             Decode ABI call result.
    abi                Load json ABI from file.
    params             Specify types of input params inline.
"#;

#[derive(Debug, RustcDecodable)]
pub struct Args {
	pub cmd_encode: bool,
	pub cmd_decode: bool,
	pub cmd_abi: bool,
	pub cmd_params: bool,
	pub arg_param: Vec<String>,
	pub arg_type: Vec<String>,
}

fn main() {
	let args: Args = Docopt::new(ETHABI)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());

	if args.cmd_encode && args.cmd_abi {
		encode_call();
	} else if args.cmd_encode && args.cmd_params {
		encode_params();
	} else if args.cmd_decode && args.cmd_abi {
		decode_call_output();
	} else if args.cmd_decode && args.cmd_params {
		decode_params();
	}
}

fn encode_call() {
}

fn encode_params() {
}

fn decode_call_output() {
}

fn decode_params() {
}
