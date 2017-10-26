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

//! Rust code contract generator.
//! The code generated will require a dependence on the `ethcore-bigint::prelude`,
//! `ethabi`, `byteorder`, and `futures` crates.
//! This currently isn't hygienic, so compilation of generated code may fail
//! due to missing crates or name collisions. This will change when
//! it can be ported to a procedural macro.

extern crate ethabi;
extern crate heck;

use ethabi::{Contract, ParamType};
use heck::SnakeCase;

/// Errors in generation.
#[derive(Debug)]
pub enum Error {
	/// Bad ABI.
	Abi(ethabi::Error),
	/// Unsupported parameter type in given function.
	UnsupportedType(String, ParamType),
}

/// Given an ABI string, generate code for a a Rust module containing
/// a struct which can be used to call it.
// TODO: make this a proc macro when that's possible.
pub fn generate_module(struct_name: &str, abi: &str) -> Result<String, Error> {
	let contract = Contract::load(abi.as_bytes()).map_err(Error::Abi)?;
	let functions = generate_functions(&contract)?;

	Ok(format!(r##"
use byteorder::{{BigEndian, ByteOrder}};
use futures::{{future, Future, IntoFuture}};
use ethabi::{{Contract, Token, Event}};
use bigint;

type BoxFuture<A, B> = Box<Future<Item = A, Error = B> + Send>;

/// Generated Rust bindings to an Ethereum contract.
#[derive(Clone, Debug)]
pub struct {name} {{
	contract: Contract,
	/// Address to make calls to.
	pub address: bigint::prelude::H160,
}}

const ABI: &'static str = r#"{abi_str}"#;

impl {name} {{
	/// Create a new instance of `{name}` with an address.
	/// Calls can be made, given a callback for dispatching calls asynchronously.
	pub fn new(address: bigint::prelude::H160) -> Self {{
		let contract = Contract::load(ABI.as_bytes())
			.expect("ABI checked at generation-time; qed");
		{name} {{
			contract: contract,
			address: address,
		}}
	}}

	/// Access the underlying `ethabi` contract.
	pub fn contract(this: &Self) -> &Contract {{
		&this.contract
	}}

	{functions}
}}
"##,
		name = struct_name,
		abi_str = abi,
		functions = functions,
	))
}

// generate function bodies from the ABI.
fn generate_functions(contract: &Contract) -> Result<String, Error> {
	let mut functions = String::new();
	for function in contract.functions() {
		let name = &function.name;
		let snake_name = name.to_snake_case();
		let inputs: Vec<_> = function.inputs.iter().map(|i| i.kind.clone()).collect();
		let outputs: Vec<_> = function.outputs.iter().map(|i| i.kind.clone()).collect();

		let (input_params, to_tokens) = input_params_codegen(&inputs)
			.map_err(|bad_type| Error::UnsupportedType(name.clone(), bad_type))?;

		let (output_type, decode_outputs) = output_params_codegen(&outputs)
			.map_err(|bad_type| Error::UnsupportedType(name.clone(), bad_type))?;

		functions.push_str(&format!(r##"
/// Call the function "{abi_name}" on the contract.
///
/// Inputs: {abi_inputs:?}
/// Outputs: {abi_outputs:?}
pub fn {snake_name}<F, U>(&self, call: F, {params}) -> BoxFuture<{output_type}, String>
	where
	    F: FnOnce(bigint::prelude::H160, Vec<u8>) -> U,
	    U: IntoFuture<Item=Vec<u8>, Error=String>,
		U::Future: Send + 'static
{{
	let function = self.contract.function(r#"{abi_name}"#)
		.expect("function existence checked at compile-time; qed").clone();
	let call_addr = self.address;

	let call_future = match function.encode_input(&{to_tokens}) {{
		Ok(call_data) => (call)(call_addr, call_data),
		Err(e) => return Box::new(future::err(format!("Error encoding call: {{:?}}", e))),
	}};

	Box::new(call_future
		.into_future()
		.and_then(move |out| function.decode_output(&out).map_err(|e| format!("{{:?}}", e)))
		.map(Vec::into_iter)
		.and_then(|mut outputs| {decode_outputs}))
}}
	"##,
		abi_name = name,
		abi_inputs = inputs,
		abi_outputs = outputs,
		snake_name = snake_name,
		params = input_params,
		output_type = output_type,
		to_tokens = to_tokens,
		decode_outputs = decode_outputs,
		))
	}

	Ok(functions)
}

// generate code for params in function signature and turning them into tokens.
//
// two pieces of code are generated: the first gives input types for the function signature,
// and the second gives code to tokenize those inputs.
//
// params of form `param_0: type_0, param_1: type_1, ...`
// tokenizing code of form `{let mut tokens = Vec::new(); tokens.push({param_X}); tokens }`
//
// returns any unsupported param type encountered.
fn input_params_codegen(inputs: &[ParamType]) -> Result<(String, String), ParamType> {
	let mut params = String::new();
	let mut to_tokens = "{ let mut tokens = Vec::new();".to_string();

	for (index, param_type) in inputs.iter().enumerate() {
		let param_name = format!("param_{}", index);
		let rust_type = rust_type(param_type.clone())?;
		let (needs_mut, tokenize_code) = tokenize(&param_name, param_type.clone());

		params.push_str(&format!("{}{}: {}, ",
			if needs_mut { "mut " } else { "" }, param_name, rust_type));

		to_tokens.push_str(&format!("tokens.push({{ {} }});", tokenize_code));
	}

	to_tokens.push_str(" tokens }");
	Ok((params, to_tokens))
}

// generate code for outputs of the function and detokenizing them.
//
// two pieces of code are generated: the first gives an output type for the function signature
// as a tuple, and the second gives code to get that tuple from a deque of tokens.
//
// produce output type of the form (type_1, type_2, ...) without trailing comma.
// produce code for getting this output type from `outputs: Vec<Token>::IntoIter`, where
// an `Err(String)` can be returned.
//
// returns any unsupported param type encountered.
fn output_params_codegen(outputs: &[ParamType]) -> Result<(String, String), ParamType> {
	let mut output_type = "(".to_string();
	let mut decode_outputs = "Ok((".to_string();

	for (index, output) in outputs.iter().cloned().enumerate() {
		let rust_type = rust_type(output.clone())?;

		output_type.push_str(&rust_type);

		decode_outputs.push_str(&format!(
			r#"
				outputs
					.next()
					.and_then(|output| {{ {} }})
					.ok_or_else(|| "Wrong output type".to_string())?
			"#,
			detokenize("output", output)
		));

		// don't append trailing commas for the last element
		// so we can reuse the same code for single-output contracts,
		// since T == (T) != (T,)
		if index < outputs.len() - 1 {
			output_type.push_str(", ");
			decode_outputs.push_str(", ");
		}
	}

	output_type.push_str(")");
	decode_outputs.push_str("))");
	Ok((output_type, decode_outputs))
}

// create code for an argument type from param type.
fn rust_type(input: ParamType) -> Result<String, ParamType> {
	Ok(match input {
		ParamType::Address => "bigint::prelude::H160".into(),
		ParamType::FixedBytes(len) if len <= 32 => format!("bigint::prelude::H{}", len * 8),
		ParamType::Bytes | ParamType::FixedBytes(_) => "Vec<u8>".into(),
		ParamType::Int(width) => match width {
			8 | 16 | 32 | 64 => format!("i{}", width),
			_ => return Err(ParamType::Int(width)),
		},
		ParamType::Uint(width) => match width {
			8 | 16 | 32 | 64 => format!("u{}", width),
			128 | 160 | 256 => format!("bigint::prelude::U{}", width),
			_ => return Err(ParamType::Uint(width)),
		},
		ParamType::Bool => "bool".into(),
		ParamType::String => "String".into(),
		ParamType::Array(kind) => format!("Vec<{}>", rust_type(*kind)?),
		other => return Err(other),
	})
}

// create code for tokenizing this parameter.
// returns (needs_mut, code), where needs_mut indicates mutability required.
// panics on unsupported types.
fn tokenize(name: &str, input: ParamType) -> (bool, String) {
	let mut needs_mut = false;
	let code = match input {
		ParamType::Address => format!("Token::Address({}.0)", name),
		ParamType::Bytes => format!("Token::Bytes({})", name),
		ParamType::FixedBytes(len) if len <= 32 =>
			format!("Token::FixedBytes({}.0.to_vec())", name),
		ParamType::FixedBytes(len) => {
			needs_mut = true;
			format!("{}.resize({}, 0); Token::FixedBytes({})", name, len, name)
		}
		ParamType::Int(width) => match width {
			8 => format!("let mut r = [0xff; 32]; r[31] = {}; Token::Int(r)", name),
			16 | 32 | 64 =>
				format!("let mut r = [0xff; 32]; BigEndian::write_i{}(&mut r[{}..], {}); Token::Int(r))",
					width, 32 - (width / 8), name),
			_ => panic!("Signed int with more than 64 bits not supported."),
		},
		ParamType::Uint(width) => format!(
			"let mut r = [0; 32]; {}.to_big_endian(&mut r); Token::Uint(r)",
			if width <= 64 { format!("bigint::prelude::U256::from({} as u64)", name) }
			else { format!("bigint::prelude::U256::from({})", name) }
		),
		ParamType::Bool => format!("Token::Bool({})", name),
		ParamType::String => format!("Token::String({})", name),
		ParamType::Array(kind) => {
			let (needs_mut, code) = tokenize("i", *kind);
			format!("Token::Array({}.into_iter().map(|{}i| {{ {} }}).collect())",
				name, if needs_mut { "mut " } else { "" }, code)
		}
		ParamType::FixedArray(_, _) => panic!("Fixed-length arrays not supported."),
	};

	(needs_mut, code)
}

// create code for detokenizing this parameter.
// takes an output type and the identifier of a token.
// expands to code that evaluates to a Option<concrete type>
// panics on unsupported types.
fn detokenize(name: &str, output_type: ParamType) -> String {
	match output_type {
		ParamType::Address => format!("{}.to_address().map(bigint::prelude::H160)", name),
		ParamType::Bytes => format!("{}.to_bytes()", name),
		ParamType::FixedBytes(len) if len <= 32 => {
			// ensure no panic on slice too small.
			let read_hash = format!("b.resize({}, 0); bigint::prelude::H{}::from_slice(&b[..{}])",
				len, len * 8, len);

			format!("{}.to_fixed_bytes().map(|mut b| {{ {} }})",
				name, read_hash)
		}
		ParamType::FixedBytes(_) => format!("{}.to_fixed_bytes()", name),
		ParamType::Int(width) => {
			let read_int = match width {
				8 => "i[31] as i8".into(),
				16 | 32 | 64 => format!("BigEndian::read_i{}(&i[{}..])", width, 32 - (width / 8)),
				_ => panic!("Signed integers over 64 bytes not allowed."),
			};
			format!("{}.to_int().map(|i| {})", name, read_int)
		}
		ParamType::Uint(width) => {
			let read_uint = match width {
				8 => "u[31] as u8".into(),
				16 | 32 | 64 => format!("BigEndian::read_u{}(&u[{}..])", width, 32 - (width / 8)),
				_ => format!("bigint::prelude::U{}::from(&u[..])", width),
			};

			format!("{}.to_uint().map(|u| {})", name, read_uint)
		}
		ParamType::Bool => format!("{}.to_bool()", name),
		ParamType::String => format!("{}.to_string()", name),
		ParamType::Array(kind) => {
			let read_array = format!("x.into_iter().map(|a| {{ {} }}).collect::<Option<Vec<_>>>()",
				detokenize("a", *kind));

			format!("{}.to_array().and_then(|x| {{ {} }})",
				name, read_array)
		}
		ParamType::FixedArray(_, _) => panic!("Fixed-length arrays not supported.")
	}
}

#[cfg(test)]
mod tests {
	use ethabi::ParamType;

	#[test]
	fn input_types() {
		assert_eq!(::input_params_codegen(&[]).unwrap().0, "");
		assert_eq!(::input_params_codegen(&[ParamType::Address]).unwrap().0, "param_0: bigint::prelude::H160, ");
		assert_eq!(::input_params_codegen(&[ParamType::Address, ParamType::Bytes]).unwrap().0,
			"param_0: bigint::prelude::H160, param_1: Vec<u8>, ");
	}

	#[test]
	fn output_types() {
		assert_eq!(::output_params_codegen(&[]).unwrap().0, "()");
		assert_eq!(::output_params_codegen(&[ParamType::Address]).unwrap().0, "(bigint::prelude::H160)");
		assert_eq!(::output_params_codegen(&[ParamType::Address, ParamType::Array(Box::new(ParamType::Bytes))]).unwrap().0,
			"(bigint::prelude::H160, Vec<Vec<u8>>)");
	}

	#[test]
	fn rust_type() {
		assert_eq!(::rust_type(ParamType::FixedBytes(32)).unwrap(), "bigint::prelude::H256");
		assert_eq!(::rust_type(ParamType::Array(Box::new(ParamType::FixedBytes(32)))).unwrap(),
			"Vec<bigint::prelude::H256>");

		assert_eq!(::rust_type(ParamType::Uint(64)).unwrap(), "u64");
		assert!(::rust_type(ParamType::Uint(63)).is_err());

		assert_eq!(::rust_type(ParamType::Int(32)).unwrap(), "i32");
		assert_eq!(::rust_type(ParamType::Uint(256)).unwrap(), "bigint::prelude::U256");
	}

	// codegen tests will need bootstrapping of some kind.
}
