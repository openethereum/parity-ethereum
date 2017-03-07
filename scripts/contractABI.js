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

// Rust/Parity ABI struct autogenerator.
// By Gav Wood, 2016.

var fs = require('fs');

String.prototype.replaceAll = function(f, t) { return this.split(f).join(t); }
String.prototype.toSnake = function(){
	return this.replace(/([A-Z])/g, function($1){return "_"+$1.toLowerCase();});
};

function makeContractFile(name, json, prefs) {
	return `// Autogenerated from JSON contract definition using Rust contract convertor.
// Command line: ${process.argv.slice(2).join(' ')}
#![allow(unused_imports)]
use std::string::String;
use std::result::Result;
use std::fmt;
use {util, ethabi};
use util::{FixedHash, Uint};

${convertContract(name, json, prefs)}
`;
}

function convertContract(name, json, prefs) {
	return `${prefs._pub ? "pub " : ""}struct ${name} {
	contract: ethabi::Contract,
	pub address: util::Address,
	${prefs._explicit_do_call ? "" : `do_call: Box<Fn(util::Address, Vec<u8>) -> Result<Vec<u8>, String> + Send${prefs._sync ? " + Sync " : ""}+ 'static>,`}
}
impl ${name} {
	pub fn new${prefs._explicit_do_call ? "" : "<F>"}(address: util::Address${prefs._explicit_do_call ? "" : `, do_call: F`}) -> Self
		${prefs._explicit_do_call ? "" : `where F: Fn(util::Address, Vec<u8>) -> Result<Vec<u8>, String> + Send ${prefs._sync ? "+ Sync " : ""}+ 'static`} {
		${name} {
			contract: ethabi::Contract::new(ethabi::Interface::load(b"${JSON.stringify(json.filter(a => a.type == 'function')).replaceAll('"', '\\"')}").expect("JSON is autogenerated; qed")),
			address: address,
			${prefs._explicit_do_call ? "" : `do_call: Box::new(do_call),`}
		}
	}
	fn as_string<T: fmt::Debug>(e: T) -> String { format!("{:?}", e) }
	${json.filter(x => x.type == 'function').map(x => convertFunction(x, prefs)).join("\n")}
}`;
}

function mapType(name, type, _prefs) {
	let prefs = _prefs || {};
	var m;
	if ((m = type.match(/^bytes(\d+)$/)) != null && m[1] <= 32) {
		if (prefs['string'])
			return `&str`;
		else
			return `&util::H${m[1] * 8}`;
	}
	if ((m = type.match(/^(u?)int(\d+)$/)) != null) {
		var n = [8, 16, 32, 64, 128, 160, 256].filter(i => m[2] <= i)[0];
		if (n) {
			if (n <= 64)
				return `${m[1] == 'u' ? 'u' : 'i'}${n}`;
			if (m[1] == 'u')
				return `util::U${n}`;
			// ERROR - unsupported integer (signed > 32 or unsigned > 256)
		}
	}
	if (type == "address")
		return "&util::Address";
	if (type == "bool")
		return "bool";
	if (type == "string")
		return "&str";
	if (type == "bytes")
		return "&[u8]";

	console.log(`Unsupported argument type: ${type} (${name})`);
}

function mapReturnType(name, type, _prefs) {
	let prefs = _prefs || {};
	var m;
	if ((m = type.match(/^bytes(\d+)$/)) != null && m[1] <= 32) {
		if (prefs['string'])
			return `String`;
		else
			return `util::H${m[1] * 8}`;
	}
	if ((m = type.match(/^(u?)int(\d+)$/)) != null) {
		var n = [8, 16, 32, 64, 128, 160, 256].filter(i => m[2] <= i)[0];
		if (n) {
			if (n <= 64)
				return `${m[1] == 'u' ? 'u' : 'i'}${n}`;
			if (m[1] == 'u')
				return `util::U${n}`;
			// ERROR - unsupported integer (signed > 32 or unsigned > 256)
		}
	}
	if (type == "address")
		return "util::Address";
	if (type == "bool")
		return "bool";
	if (type == "string")
		return "String";
	if (type == "bytes")
		return "Vec<u8>";
	if (type == "address[]")
		return "Vec<util::Address>";

	console.log(`Unsupported return type: ${type} (${name})`);
}

function convertToken(name, type, _prefs) {
	let prefs = _prefs || {};
	var m;
	if ((m = type.match(/^bytes(\d+)$/)) != null && m[1] <= 32) {
		if (prefs['string'])
			return `ethabi::Token::FixedBytes(${name}.as_bytes().to_owned())`;
		else
			return `ethabi::Token::FixedBytes(${name}.as_ref().to_owned())`;
	}
	if ((m = type.match(/^(u?)int(\d+)$/)) != null) {
		var n = [8, 16, 32, 64, 128, 160, 256].filter(i => m[2] <= i)[0];
		if (n) {
			if (m[1] == 'u')
				return `ethabi::Token::Uint({ let mut r = [0u8; 32]; ${n <= 64 ? "util::U256::from(" + name + " as u64)" : name}.to_big_endian(&mut r); r })`;
			else if (n <= 32)
				return `ethabi::Token::Int(pad_i32(${name} as i32))`;
			// ERROR - unsupported integer (signed > 32 or unsigned > 256)
		}
	}
	if (type == "address")
		return `ethabi::Token::Address(${name}.clone().0)`;
	if (type == "bool")
		return `ethabi::Token::Bool(${name})`;
	if (type == "string")
		return `ethabi::Token::String(${name}.to_owned())`;
	if (type == "bytes")
		return `ethabi::Token::Bytes(${name}.to_owned())`;

	console.log(`Unsupported argument type: ${type} (${name})`);
}

function tokenType(name, type, _prefs) {
	let prefs = _prefs || {};
	var m;
	if ((m = type.match(/^bytes(\d+)$/)) != null && m[1] <= 32)
		return `${name}.to_fixed_bytes()`;
	if ((m = type.match(/^(u?)int(\d+)$/)) != null) {
		return `${name}.to_${m[1]}int()`;
	}
	if (type == "address")
		return `${name}.to_address()`;
	if (type == "bool")
		return `${name}.to_bool()`;
	if (type == "string")
		return `${name}.to_string()`;
	if (type == "bytes")
		return `${name}.to_bytes()`;
	if (type == "address[]")
		return `${name}.to_array().and_then(|v| v.into_iter().map(|a| a.to_address()).collect::<Option<Vec<[u8; 20]>>>())`;

	console.log(`Unsupported return type: ${type} (${name})`);
}

function tokenCoerce(name, type, _prefs) {
	let prefs = _prefs || {};
	var m;
	if ((m = type.match(/^bytes(\d+)$/)) != null && m[1] <= 32) {
		if (prefs['string'])
			return `String::from_utf8(${name}).unwrap_or_else(String::new)`;
		else
			return `util::H${m[1] * 8}::from_slice(${name}.as_ref())`;
	}
	if ((m = type.match(/^(u?)int(\d+)$/)) != null) {
		var n = [8, 16, 32, 64, 128, 160, 256].filter(i => m[2] <= i)[0];
		if (n && m[1] == 'u')
			return `util::U${n <= 64 ? 256 : n}::from(${name}.as_ref())` + (n <= 64 ? `.as_u64() as u${n}` : '');
		// ERROR - unsupported integer (signed or unsigned > 256)
	}
	if (type == "address")
		return `util::Address::from(${name})`;
	if (type == "bool")
		return `${name}`;
	if (type == "string")
		return `${name}`;
	if (type == "bytes")
		return `${name}`;
	if (type == "address[]")
		return `${name}.into_iter().map(|a| util::Address::from(a)).collect::<Vec<_>>()`;

	console.log(`Unsupported return type: ${type} (${name})`);
}

function tokenExtract(expr, type, _prefs) {
	return `{ let r = ${expr}; let r = ${tokenType('r', type, _prefs)}.ok_or("Invalid type returned")?; ${tokenCoerce('r', type, _prefs)} }`;
}

function convertFunction(json, _prefs) {
	let cprefs = _prefs || {};
	let prefs = (_prefs || {})[json.name] || (_prefs || {})['_'] || {};
	let snakeName = json.name.toSnake();
	let params = json.inputs.map((x, i) => (x.name ? x.name.toSnake() : ("_" + (i + 1))) + ": " + mapType(x.name, x.type, prefs[x.name]));
	let returns = json.outputs.length != 1 ? "(" + json.outputs.map(x => mapReturnType(x.name, x.type, prefs[x.name])).join(", ") + ")" : mapReturnType(json.outputs[0].name, json.outputs[0].type, prefs[json.outputs[0].name]);
	return `
	/// Auto-generated from: \`${JSON.stringify(json)}\`
	#[allow(dead_code)]
	pub fn ${snakeName}${cprefs._explicit_do_call ? "<F>" : ""}(&self${cprefs._explicit_do_call ? `, do_call: &F` : ""}${params.length > 0 ? ', ' + params.join(", ") : ''}) -> Result<${returns}, String>
		${cprefs._explicit_do_call ? `where F: Fn(util::Address, Vec<u8>) -> Result<Vec<u8>, String> + Send ${prefs._sync ? "+ Sync " : ""}` : ""} {
		let call = self.contract.function("${json.name}".into()).map_err(Self::as_string)?;
		let data = call.encode_call(
			vec![${json.inputs.map((x, i) => convertToken(x.name ? x.name.toSnake() : ("_" + (i + 1)), x.type, prefs[x.name])).join(', ')}]
		).map_err(Self::as_string)?;
		${json.outputs.length > 0 ? 'let output = ' : ''}call.decode_output((${cprefs._explicit_do_call ? "" : "self."}do_call)(self.address.clone(), data)?).map_err(Self::as_string)?;
		${json.outputs.length > 0 ? 'let mut result = output.into_iter().rev().collect::<Vec<_>>();' : ''}
		Ok((${json.outputs.map((o, i) => tokenExtract('result.pop().ok_or("Invalid return arity")?', o.type, prefs[o.name])).join(', ')}))
	}`;
}

// default preferences:
let prefs = {"_pub": true, "_": {"_client": {"string": true}, "_platform": {"string": true}}, "_sync": true};
// default contract json ABI
let jsonabi = [{"constant":true,"inputs":[],"name":"getValidators","outputs":[{"name":"","type":"address[]"}],"payable":false,"type":"function"}];
// default name
let name = 'Contract';

// parse command line options
for (let i = 1; i < process.argv.length; ++i) {
	let arg = process.argv[i];
	if (arg.indexOf("--jsonabi=") == 0) {
		jsonabi = arg.slice(10);
		if (fs.existsSync(jsonabi)) {
			jsonabi = JSON.parse(fs.readFileSync(jsonabi).toString());
		}
	} else if (arg.indexOf("--explicit-do-call") == 0) {
		prefs._explicit_do_call = true;
	} else if (arg.indexOf("--name=") == 0) {
		name = arg.slice(7);
	}
}

let out = makeContractFile(name, jsonabi, prefs);
console.log(`${out}`);
