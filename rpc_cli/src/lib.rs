extern crate futures;

extern crate ethcore_util as util;
extern crate ethcore_rpc as rpc;
extern crate ethcore_bigint as bigint;
extern crate rpassword;

extern crate parity_rpc_client as client;

use rpc::v1::types::{U256, ConfirmationRequest};
use client::signer::SignerRpc;
use std::io::{Write, BufRead, BufReader, stdout, stdin};
use std::path::PathBuf;
use std::fs::File;

use futures::Future;

fn sign_interactive(signer: &mut SignerRpc, pwd: &String, request: ConfirmationRequest)
					-> Result<String, String>
{
	print!("\n{}\nSign this transaction? (y)es/(N)o/(r)eject: ", request);
	stdout().flush();
	match BufReader::new(stdin()).lines().next() {
		Some(Ok(line)) => {
			match line.to_lowercase().chars().nth(0) {
				Some('y') => {
					match sign_transaction(signer, request.id, pwd) {
						Ok(s) | Err(s) => println!("{}", s),
					}
				}
				Some('r') => {
					match reject_transaction(signer, request.id) {
						Ok(s) | Err(s) => println!("{}", s),
					}
				}
				_ => ()
			}
		}
		_ => return Err("Could not read from stdin".to_string())
	}
	Ok("Finished".to_string())
}

fn sign_transactions(signer: &mut SignerRpc, pwd: String) -> Result<String, String> {
	signer.requests_to_confirm().map(|reqs| {
		match reqs {
			Ok(reqs) => {
				if reqs.len() == 0 {
					Ok("No transactions in signing queue".to_string())
				} else {
					for r in reqs {
						sign_interactive(signer, &pwd, r);
					}
					Ok("".to_string())
				}
			}
			Err(err) => {
				Err(format!("error: {:?}", err))
			}
		}
	}).wait().unwrap()
}

fn list_transactions(signer: &mut SignerRpc) -> Result<String, String> {
	signer.requests_to_confirm().map(|reqs| {
		match reqs {
			Ok(reqs) => {
				let mut s = "Transaction queue:".to_string();
				if reqs.len() == 0 {
					s = s + &"No transactions in signing queue";
				} else {
					for r in reqs {
						s = s + &format!("\n{}", r);
					}
				}
				Ok(s)
			}
			Err(err) => {
				Err(format!("error: {:?}", err))
			}
		}
	}).wait().unwrap()
}

fn sign_transaction(signer: &mut SignerRpc,
					id: U256,
					pwd: &String) -> Result<String, String> {
	signer.confirm_request(id, None, &pwd).map(|res| {
		match res {
			Ok(u) => Ok(format!("Signed transaction id: {:#x}", u)),
			Err(e) => Err(format!("{:?}", e)),
		}
	}).wait().unwrap()
}

fn reject_transaction(signer: &mut SignerRpc,
					  id: U256) -> Result<String, String> {
	signer.reject_request(id).map(|res| {
		match res {
			Ok(true) => Ok(format!("Rejected transaction id {:#x}", id)),
			Ok(false) => Err(format!("No such request")),
			Err(e) => Err(format!("{:?}", e)),
		}
	}).wait().unwrap()
}

// cmds

pub fn cmd_signer_list(signerport: u16,
					   authfile: PathBuf) -> Result<String, String> {
	match SignerRpc::new(&format!("ws://127.0.0.1:{}", signerport),
						 &authfile) {
		Ok(mut signer) => {
			list_transactions(&mut signer)
		}
		Err(e) => Err(format!("{:?}", e))
	}
}

pub fn cmd_signer_reject(id: usize,
						 signerport: u16,
						 authfile: PathBuf) -> Result<String, String> {
	match SignerRpc::new(&format!("ws://127.0.0.1:{}", signerport),
						 &authfile) {
		Ok(mut signer) => {
			reject_transaction(&mut signer, U256::from(id))
		},
		Err(e) => Err(format!("{:?}", e))
	}
}

pub fn cmd_signer_sign(id: Option<usize>,
					   pwfile: Option<PathBuf>,
					   signerport: u16,
					   authfile: PathBuf) -> Result<String, String> {
	let pwd;
	match pwfile {
		Some(pwfile) => {
			match File::open(pwfile) {
				Ok(fd) => {
					match BufReader::new(fd).lines().next() {
						Some(Ok(line)) => pwd = line,
						_ => return Err(format!("No password in file"))
					}
				},
				Err(e) => return Err(format!("Could not open pwfile: {}", e))
			}
		}
		None => {
			pwd = rpassword::prompt_password_stdout("Password: ").unwrap();
		}
	}

	match SignerRpc::new(&format!("ws://127.0.0.1:{}", signerport),
						 &authfile) {
		Ok(mut signer) => {
			match id {
				Some(id) => {
					sign_transaction(&mut signer, U256::from(id), &pwd)
				},
				None => {
					sign_transactions(&mut signer, pwd)
				}
			}
		}
		Err(e) => return Err(format!("{:?}", e))
	}
}
