use json;

#[derive(Debug, PartialEq, Clone)]
pub struct Aes128Ctr {
	pub iv: [u8; 16],
}

#[derive(Debug, PartialEq, Clone)]
pub enum Cipher {
	Aes128Ctr(Aes128Ctr),
}

impl From<json::Aes128Ctr> for Aes128Ctr {
	fn from(json: json::Aes128Ctr) -> Self {
		Aes128Ctr {
			iv: json.iv.into()
		}
	}
}

impl Into<json::Aes128Ctr> for Aes128Ctr {
	fn into(self) -> json::Aes128Ctr {
		json::Aes128Ctr {
			iv: From::from(self.iv)
		}
	}
}

impl From<json::Cipher> for Cipher {
	fn from(json: json::Cipher) -> Self {
		match json {
			json::Cipher::Aes128Ctr(params) => Cipher::Aes128Ctr(From::from(params)),
		}
	}
}

impl Into<json::Cipher> for Cipher {
	fn into(self) -> json::Cipher {
		match self {
			Cipher::Aes128Ctr(params) => json::Cipher::Aes128Ctr(params.into()),
		}
	}
}
