use util::*;
use pod_account::*;

#[derive(Debug,Clone,PartialEq,Eq,Default)]
/// TODO [Gav Wood] Please document me
pub struct PodState (BTreeMap<Address, PodAccount>);

impl PodState {
	/// Contruct a new object from the `m`.
	pub fn new() -> PodState { Default::default() }

	/// Contruct a new object from the `m`.
	pub fn from(m: BTreeMap<Address, PodAccount>) -> PodState { PodState(m) }

	/// Get the underlying map.
	pub fn get(&self) -> &BTreeMap<Address, PodAccount> { &self.0 }

	/// Get the root hash of the trie of the RLP of this.
	pub fn root(&self) -> H256 {
		sec_trie_root(self.0.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect())
	}

	/// Drain object to get the underlying map.
	pub fn drain(self) -> BTreeMap<Address, PodAccount> { self.0 }
}

impl FromJson for PodState {
	/// Translate the JSON object into a hash map of account information ready for insertion into State.
	fn from_json(json: &Json) -> PodState {
		PodState(json.as_object().unwrap().iter().fold(BTreeMap::new(), |mut state, (address, acc)| {
			let balance = acc.find("balance").map(&U256::from_json);
			let nonce = acc.find("nonce").map(&U256::from_json);
			let storage = acc.find("storage").map(&BTreeMap::from_json);
			let code = acc.find("code").map(&Bytes::from_json);
			if balance.is_some() || nonce.is_some() || storage.is_some() || code.is_some() {
				state.insert(address_from_hex(address), PodAccount{
					balance: balance.unwrap_or_else(U256::zero),
					nonce: nonce.unwrap_or_else(U256::zero),
					storage: storage.unwrap_or_else(BTreeMap::new),
					code: code.unwrap_or_else(Vec::new)
				});
			}
			state
		}))
	}
}

impl fmt::Display for PodState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (add, acc) in &self.0 {
			try!(writeln!(f, "{} => {}", add, acc));
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	extern crate rustc_serialize;

	use super::*;
	use rustc_serialize::*;
	use util::from_json::FromJson;
	use util::hash::*;

	#[test]
	fn it_serializes_form_json() {
		let pod_state = PodState::from_json(&json::Json::from_str(
r#"
	{
		"0000000000000000000000000000000000000000": {
			"balance": "1000",
			"nonce": "100",
			"storage": {},
			"code" : []
		}
	}
"#
		).unwrap());

		assert!(pod_state.get().get(&ZERO_ADDRESS).is_some());
	}
}
