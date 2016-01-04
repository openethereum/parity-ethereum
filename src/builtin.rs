use std::cmp::min;
use util::uint::*;
use rustc_serialize::json::Json;

/// Definition of a contract whose implementation is built-in. 
pub struct Builtin {
	/// The gas cost of running this built-in for the given size of input data.
	pub cost: Box<Fn(usize) -> U256>,	// TODO: U256 should be bignum.
	/// Run this built-in function with the input being the first argument and the output
	/// being placed into the second.
	pub execute: Box<Fn(&[u8], &mut [u8])>,
}

/*
ETH_REGISTER_PRECOMPILED(ecrecover)(bytesConstRef _in, bytesRef _out)
{
	struct inType
	{
		h256 hash;
		h256 v;
		h256 r;
		h256 s;
	} in;

	memcpy(&in, _in.data(), min(_in.size(), sizeof(in)));

	h256 ret;
	u256 v = (u256)in.v;
	if (v >= 27 && v <= 28)
	{
		SignatureStruct sig(in.r, in.s, (byte)((int)v - 27));
		if (sig.isValid())
		{
			try
			{
				if (Public rec = recover(sig, in.hash))
				{
					ret = dev::sha3(rec);
					memset(ret.data(), 0, 12);
					ret.ref().copyTo(_out);
				}
			}
			catch (...) {}
		}
	}
}

ETH_REGISTER_PRECOMPILED(sha256)(bytesConstRef _in, bytesRef _out)
{
	dev::sha256(_in).ref().copyTo(_out);
}

ETH_REGISTER_PRECOMPILED(ripemd160)(bytesConstRef _in, bytesRef _out)
{
	h256(dev::ripemd160(_in), h256::AlignRight).ref().copyTo(_out);
}
*/

impl Builtin {
	/// Create a new object from components.
	pub fn new(cost: Box<Fn(usize) -> U256>, execute: Box<Fn(&[u8], &mut [u8])>) -> Builtin {
		Builtin {cost: cost, execute: execute}
	}

	/// Create a new object from a builtin-function name with a linear cost associated with input size.
	pub fn from_named_linear(name: &str, base_cost: usize, word_cost: usize) -> Option<Builtin> {
		new_builtin_exec(name).map(|b| {
			let cost = Box::new(move|s: usize| -> U256 {U256::from(base_cost) + U256::from(word_cost) * U256::from((s + 31) / 32)});
			Self::new(cost, b)
		})
	}

	/// Create a builtin from JSON.
	///
	/// JSON must be of the form `{ "name": "identity", "linear": {"base": 10, "word": 20} }`.
	pub fn from_json(json: &Json) -> Option<Builtin> {
		// NICE: figure out a more convenient means of handing errors here.
		if let Json::String(ref name) = json["name"] {
			if let Json::Object(ref o) = json["linear"] {
				if let Json::U64(ref word) = o["word"] {
					if let Json::U64(ref base) = o["base"] {
						return Self::from_named_linear(&name[..], *base as usize, *word as usize);
					}
				}
			}
		}
		None
	}
}

// TODO: turn in to a factory with dynamic registration.
pub fn new_builtin_exec(name: &str) -> Option<Box<Fn(&[u8], &mut [u8])>> {
	match name {
		"identity" => Some(Box::new(move|input: &[u8], output: &mut[u8]| {
			for i in 0..min(input.len(), output.len()) {
				output[i] = input[i];
			}
		})),
		"ecrecover" => Some(Box::new(move|_input: &[u8], _output: &mut[u8]| {
			unimplemented!();
		})),
		"sha256" => Some(Box::new(move|_input: &[u8], _output: &mut[u8]| {
			unimplemented!();
		})),
		"ripemd160" => Some(Box::new(move|_input: &[u8], _output: &mut[u8]| {
			unimplemented!();
		})),
		_ => None
	}
}

#[test]
fn identity() {
	let f = new_builtin_exec("identity").unwrap();
	let i = [0u8, 1, 2, 3];
	for osize in [2, 4, 8].iter() {
		let mut o = vec![255u8; *osize];
		f(&i[..], &mut o[..]);
		assert_eq!(i[..], o[..]);
	}
}