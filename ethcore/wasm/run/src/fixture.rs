use std::borrow::Cow;
use ethjson::uint::Uint;
use ethjson::hash::{Address, H256};
use ethjson::bytes::Bytes;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Source {
	Raw(Cow<'static, String>),
	Constructor {
		#[serde(rename="constructor")]
		source: Cow<'static, String>,
		arguments: Bytes,
		sender: Address,
		at: Address,
	},
}

impl Source {
	pub fn as_ref(&self) -> &str {
		match *self {
			Source::Raw(ref r) => r.as_ref(),
			Source::Constructor { ref source, .. } => source.as_ref(),
		}
	}
}

#[derive(Deserialize)]
pub struct Fixture {
	pub caption: Cow<'static, String>,
	pub source: Source,
	pub address: Option<Address>,
	pub sender: Option<Address>,
	pub value: Option<Uint>,
	#[serde(rename="gasLimit")]
	pub gas_limit: Option<u64>,
	pub payload: Option<Bytes>,
	pub storage: Option<Vec<StorageEntry>>,
	pub asserts: Vec<Assert>,
}

#[derive(Deserialize, Debug)]
pub struct StorageEntry {
	pub key: Uint,
	pub value: Uint,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CallLocator {
	pub sender: Option<Address>,
	pub receiver: Option<Address>,
	pub value: Option<Uint>,
	pub data: Option<Bytes>,
	#[serde(rename="codeAddress")]
	pub code_address: Option<Address>,
}

#[derive(Deserialize, Debug)]
pub struct StorageAssert {
	pub key: H256,
	pub value: H256,
}

#[derive(Deserialize, Debug)]
pub enum Assert {
	HasCall(CallLocator),
	HasStorage(StorageAssert),
	UsedGas(u64),
	Return(Bytes),
}