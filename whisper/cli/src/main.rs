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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parity_whisper;
extern crate panic_hook;
extern crate ethcore_network as net;

use std::{env, fmt, process};
use docopt::Docopt;
use std::io;
use net::*;


use std::sync::Arc; // no
use parity_whisper::net::{self as whisper_net, Network as WhisperNetwork};
use parity_whisper::rpc::{WhisperClient, FilterManager};


pub const USAGE: &'static str = r#"
Whisper.
  Copyright 2017 Parity Technologies (UK) Ltd

Usage:
	whisper [options]
    whisper [-h | --help]

Options:
    --whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
    -h, --help                     Display this message and exit.
"#;

// go to clap

/*


Commands:
    generate           Generates new ethereum key.
    random             Random generation.
    prefix             Random generation, but address must start with a prefix
    brain              Generate new key from string seed.
    sign               Sign message using secret.
    verify             Verify signer of the signature.
	*/

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
}

#[derive(Debug)]
enum Error {
	// Whisper(WhisperError),
	Docopt(docopt::Error),
	Io(io::Error),
}

// impl From<WhisperError> for Error {
// 	fn from(err: WhisperError) -> Self {
// 		Error::Whisper(err)
// 	}
// }

impl From<docopt::Error> for Error {
	fn from(err: docopt::Error) -> Self {
		Error::Docopt(err)
	}
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Error::Io(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			// Error::Whisper(ref e) => write!(f, "{}", e),
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
		}
	}
}

fn main() {
	panic_hook::set();

	match execute(env::args()) {
		Ok(ok) => println!("{}", ok),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		},
	}
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).deserialize())?;

	let target_message_pool_size = args.flag_whisper_pool_size * 1024 * 1024;



	let manager = Arc::new(FilterManager::new()?);
	let whisper_handler = Arc::new(WhisperNetwork::new(target_message_pool_size, manager.clone()));



	let mut service = NetworkService::new(NetworkConfiguration::new_local(), None).expect("Error creating network service");
	service.start().expect("Error starting service");
	service.register_protocol(whisper_handler, whisper_net::PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);
	service.register_protocol(Arc::new(whisper_net::ParityExtensions), whisper_net::PARITY_PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);
	// Arc::new(whisper_net::ParityExtensions),
	// ou bien via factory




	/*
		util/network/src/networkconfigration
		create netwokconfiguration, crete network ervice (lib.rs)
	*/

	// let (whisper_net, whisper_factory) = ::whisper::setup(target_message_pool_size)
	// 	.map_err(|e| format!("Failed to initialize whisper: {}", e))?;

	// attached_protos.push(whisper_net);

	Ok("OK".to_owned())
}










// attached protos est vide au début
// pub fn setup(target_pool_size: usize, protos: &mut Vec<AttachedProtocol>)
// 	-> io::Result<Option<RpcFactory>>
// {
// 	let manager = Arc::new(FilterManager::new()?);
// 	let net = Arc::new(WhisperNetwork::new(target_pool_size, manager.clone()));

// 	protos.push(AttachedProtocol {
// 		handler: net.clone() as Arc<_>,
// 		packet_count: whisper_net::PACKET_COUNT,
// 		versions: whisper_net::SUPPORTED_VERSIONS,
// 		protocol_id: whisper_net::PROTOCOL_ID,
// 	});

// 	// parity-only extensions to whisper.
// 	protos.push(AttachedProtocol {
// 		handler: Arc::new(whisper_net::ParityExtensions),
// 		packet_count: whisper_net::PACKET_COUNT,
// 		versions: whisper_net::SUPPORTED_VERSIONS,
// 		protocol_id: whisper_net::PARITY_PROTOCOL_ID,
// 	});

// 	let factory = RpcFactory { net: net, manager: manager };

// 	Ok(Some(factory))
// }





