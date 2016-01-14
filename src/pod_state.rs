use util::*;
use pod_account::*;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PodState (BTreeMap<Address, PodAccount>);

impl PodState {
	/// Contruct a new object from the `m`.
	pub fn new(m: BTreeMap<Address, PodAccount>) -> PodState { PodState(m) }

	/// Get the underlying map.
	pub fn get(&self) -> &BTreeMap<Address, PodAccount> { &self.0 }

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
					balance: balance.unwrap_or(U256::zero()),
					nonce: nonce.unwrap_or(U256::zero()),
					storage: storage.unwrap_or(BTreeMap::new()),
					code: code.unwrap_or(Vec::new())
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
