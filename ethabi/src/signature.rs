use tiny_keccak::Keccak;
use spec::ParamType;
use spec::param_type::Writer;

pub fn signature(name: &str, params: &[ParamType]) -> Vec<u8> {
	let types = params.iter()
		.map(Writer::write)
		.collect::<Vec<String>>()
		.join(",");

	let data: Vec<u8> = From::from(format!("{}({})", name, types).as_str());
	let mut result = [0u8; 4];

	let mut sponge = Keccak::new_keccak256();
	sponge.update(&data);
	sponge.finalize(&mut result);
	result.to_vec()
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use spec::ParamType;
	use super::signature;

	#[test]
	fn test_signature() {
		assert_eq!("cdcd77c0".from_hex().unwrap(), signature("baz", &[ParamType::Uint(32), ParamType::Bool]));
	}
}
