use rustc_serialize::hex::FromHex;

pub fn hex_or_string(s: &str) -> Vec<u8> {
	match s.starts_with("0x") {
		true => s[2..].from_hex().unwrap(),
		false => From::from(s)
	}
}
