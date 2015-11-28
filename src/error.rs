#![feature(concat_idents)]

use rustc_serialize::hex::*;

#[derive(Debug)]
pub enum BaseDataError {
	NegativelyReferencedHash,
}

#[derive(Debug)]
pub enum EthcoreError {
	FromHex(FromHexError),
	BaseData(BaseDataError),
	BadSize,
}

impl From<FromHexError> for EthcoreError {
	fn from(err: FromHexError) -> EthcoreError {
		EthcoreError::FromHex(err)
	}
}

impl From<BaseDataError> for EthcoreError {
	fn from(err: BaseDataError) -> EthcoreError {
		EthcoreError::BaseData(err)
	}
}

// TODO: uncomment below once https://github.com/rust-lang/rust/issues/27336 sorted.
/*macro_rules! assimilate {
    ($name:ident) => (
		impl From<concat_idents!($name, Error)> for EthcoreError {
			fn from(err: concat_idents!($name, Error)) -> EthcoreError {
				EthcoreError:: $name (err)
			}
		}
    )
}
assimilate!(FromHex);
assimilate!(BaseData);*/