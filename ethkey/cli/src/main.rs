// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

extern crate docopt;
extern crate rustc_serialize;
extern crate ethkey;

use std::{env, fmt, process};
use std::num::ParseIntError;
use docopt::Docopt;
use rustc_serialize::hex::{FromHex, FromHexError};
use ethkey::{KeyPair, Random, Brain, Prefix, Error as EthkeyError, Generator, sign, verify_public, verify_address};

pub const USAGE: &'static str = r#"
Ethereum keys generator.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    ethkey info <secret> [options]
    ethkey generate random [options]
    ethkey generate prefix <prefix> <iterations> [options]
    ethkey generate brain <seed> [options]
    ethkey sign <secret> <message>
    ethkey verify public <public> <signature> <message>
    ethkey verify address <address> <signature> <message>
    ethkey [-h | --help]

Options:
    -h, --help         Display this message and exit.
    -s, --secret       Display only the secret.
    -p, --public       Display only the public.
    -a, --address      Display only the address.

Commands:
    info               Display public and address of the secret.
    generate           Generates new ethereum key.
    random             Random generation.
    prefix             Random generation, but address must start with a prefix
    brain              Generate new key from string seed.
    sign               Sign message using secret.
    verify             Verify signer of the signature.
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
	cmd_info: bool,
	cmd_generate: bool,
	cmd_random: bool,
	cmd_prefix: bool,
	cmd_brain: bool,
	cmd_sign: bool,
	cmd_verify: bool,
	cmd_public: bool,
	cmd_address: bool,
	arg_prefix: String,
	arg_iterations: String,
	arg_seed: String,
	arg_secret: String,
	arg_message: String,
	arg_public: String,
	arg_address: String,
	arg_signature: String,
	flag_secret: bool,
	flag_public: bool,
	flag_address: bool,
}

#[derive(Debug)]
enum Error {
	Ethkey(EthkeyError),
	FromHex(FromHexError),
	ParseInt(ParseIntError),
}

impl From<EthkeyError> for Error {
	fn from(err: EthkeyError) -> Self {
		Error::Ethkey(err)
	}
}

impl From<FromHexError> for Error {
	fn from(err: FromHexError) -> Self {
		Error::FromHex(err)
	}
}

impl From<ParseIntError> for Error {
	fn from(err: ParseIntError) -> Self {
		Error::ParseInt(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::Ethkey(ref e) => write!(f, "{}", e),
			Error::FromHex(ref e) => write!(f, "{}", e),
			Error::ParseInt(ref e) => write!(f, "{}", e),
		}
	}
}

enum DisplayMode {
	KeyPair,
	Secret,
	Public,
	Address,
}

impl DisplayMode {
	fn new(args: &Args) -> Self {
		if args.flag_secret {
			DisplayMode::Secret
		} else if args.flag_public {
			DisplayMode::Public
		} else if args.flag_address {
			DisplayMode::Address
		} else {
			DisplayMode::KeyPair
		}
	}
}

fn main() {
	match execute(env::args()) {
		Ok(ok) => println!("{}", ok),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		},
	}
}

fn display(keypair: KeyPair, mode: DisplayMode) -> String {
	match mode {
		DisplayMode::KeyPair => format!("{}", keypair),
		DisplayMode::Secret => format!("{:?}", keypair.secret()),
		DisplayMode::Public => format!("{:?}", keypair.public()),
		DisplayMode::Address => format!("{:?}", keypair.address()),
	}
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).decode())
		.unwrap_or_else(|e| e.exit());

	return if args.cmd_info {
		let display_mode = DisplayMode::new(&args);
		let secret = args.arg_secret.parse().map_err(|_| EthkeyError::InvalidSecret)?;
		let keypair = KeyPair::from_secret(secret)?;
		Ok(display(keypair, display_mode))
	} else if args.cmd_generate {
		let display_mode = DisplayMode::new(&args);
		let keypair = if args.cmd_random {
			Random.generate()
		} else if args.cmd_prefix {
			let prefix = args.arg_prefix.from_hex()?;
			let iterations = usize::from_str_radix(&args.arg_iterations, 10)?;
			Prefix::new(prefix, iterations).generate()
		} else if args.cmd_brain {
			Brain::new(args.arg_seed).generate()
		} else {
			unreachable!();
		};
		Ok(display(keypair?, display_mode))
	} else if args.cmd_sign {
		let secret = args.arg_secret.parse().map_err(|_| EthkeyError::InvalidSecret)?;
		let message = args.arg_message.parse().map_err(|_| EthkeyError::InvalidMessage)?;
		let signature = sign(&secret, &message)?;
		Ok(format!("{}", signature))
	} else if args.cmd_verify {
		let signature = args.arg_signature.parse().map_err(|_| EthkeyError::InvalidSignature)?;
		let message = args.arg_message.parse().map_err(|_| EthkeyError::InvalidMessage)?;
		let ok = if args.cmd_public {
			let public = args.arg_public.parse().map_err(|_| EthkeyError::InvalidPublic)?;
			verify_public(&public, &signature, &message)?
		} else if args.cmd_address {
			let address = args.arg_address.parse().map_err(|_| EthkeyError::InvalidAddress)?;
			verify_address(&address, &signature, &message)?
		} else {
			unreachable!();
		};
		Ok(format!("{}", ok))
	} else {
		unreachable!();
	}
}

#[cfg(test)]
mod tests {
	use super::execute;

	#[test]
	fn info() {
		let command = vec!["ethkey", "info", "17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected =
"secret:  17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55
public:  689268c0ff57a20cd299fa60d3fb374862aff565b20b5f1767906a99e6e09f3ff04ca2b2a5cd22f62941db103c0356df1a8ed20ce322cab2483db67685afd124
address: 26d1ec50b4e62c1d1a40d16e7cacc6a6580757d5".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn brain() {
		let command = vec!["ethkey", "generate", "brain", "this is sparta"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected =
"secret:  17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55
public:  689268c0ff57a20cd299fa60d3fb374862aff565b20b5f1767906a99e6e09f3ff04ca2b2a5cd22f62941db103c0356df1a8ed20ce322cab2483db67685afd124
address: 26d1ec50b4e62c1d1a40d16e7cacc6a6580757d5".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn secret() {
		let command = vec!["ethkey", "generate", "brain", "this is sparta", "--secret"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn public() {
		let command = vec!["ethkey", "generate", "brain", "this is sparta", "--public"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "689268c0ff57a20cd299fa60d3fb374862aff565b20b5f1767906a99e6e09f3ff04ca2b2a5cd22f62941db103c0356df1a8ed20ce322cab2483db67685afd124".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn address() {
		let command = vec!["ethkey", "generate", "brain", "this is sparta", "--address"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "26d1ec50b4e62c1d1a40d16e7cacc6a6580757d5".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn sign() {
		let command = vec!["ethkey", "sign", "17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55", "bd50b7370c3f96733b31744c6c45079e7ae6c8d299613246d28ebcef507ec987"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "c1878cf60417151c766a712653d26ef350c8c75393458b7a9be715f053215af63dfd3b02c2ae65a8677917a8efa3172acb71cb90196e42106953ea0363c5aaf200".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn verify_valid_public() {
		let command = vec!["ethkey", "verify", "public", "689268c0ff57a20cd299fa60d3fb374862aff565b20b5f1767906a99e6e09f3ff04ca2b2a5cd22f62941db103c0356df1a8ed20ce322cab2483db67685afd124", "c1878cf60417151c766a712653d26ef350c8c75393458b7a9be715f053215af63dfd3b02c2ae65a8677917a8efa3172acb71cb90196e42106953ea0363c5aaf200", "bd50b7370c3f96733b31744c6c45079e7ae6c8d299613246d28ebcef507ec987"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "true".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn verify_valid_address() {
		let command = vec!["ethkey", "verify", "address", "26d1ec50b4e62c1d1a40d16e7cacc6a6580757d5", "c1878cf60417151c766a712653d26ef350c8c75393458b7a9be715f053215af63dfd3b02c2ae65a8677917a8efa3172acb71cb90196e42106953ea0363c5aaf200", "bd50b7370c3f96733b31744c6c45079e7ae6c8d299613246d28ebcef507ec987"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "true".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn verify_invalid() {
		let command = vec!["ethkey", "verify", "public", "689268c0ff57a20cd299fa60d3fb374862aff565b20b5f1767906a99e6e09f3ff04ca2b2a5cd22f62941db103c0356df1a8ed20ce322cab2483db67685afd124", "c1878cf60417151c766a712653d26ef350c8c75393458b7a9be715f053215af63dfd3b02c2ae65a8677917a8efa3172acb71cb90196e42106953ea0363c5aaf200", "bd50b7370c3f96733b31744c6c45079e7ae6c8d299613246d28ebcef507ec986"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let expected = "false".to_owned();
		assert_eq!(execute(command).unwrap(), expected);
	}
}
