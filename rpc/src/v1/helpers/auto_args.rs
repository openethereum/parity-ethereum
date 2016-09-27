// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Automatically serialize and deserialize parameters around a strongly-typed function.

// because we reuse the type names as idents in the macros as a dirty hack to
// work around `concat_idents!` being unstable.
#![allow(non_snake_case)]

use super::errors;

use jsonrpc_core::{Error, Params, Value, from_params, to_value};
use serde::{Serialize, Deserialize};

/// Auto-generates an RPC trait from trait definition.
///
/// This just copies out all the methods, docs, and adds another
/// function `to_delegate` which will automatically wrap each strongly-typed
/// function in a wrapper which handles parameter and output type serialization.
///
/// Every function must have a `#[name("rpc_nameHere")]` attribute after
/// its documentation, and no other attributes. All function names are
/// allowed except for `to_delegate`, which is auto-generated.
macro_rules! build_rpc_trait {
	(
		$(#[$t_attr: meta])*
		pub trait $name: ident {
			$(
				$(#[doc=$m_doc: expr])* #[name($rpc_name: expr)]
				fn $method: ident (&self $(, $param: ty)*) -> $out: ty;
			)*
		}
	) => {
		$(#[$t_attr])*
		pub trait $name: Sized + Send + Sync + 'static {
			$(
				$(#[doc=$m_doc])*
				fn $method(&self $(, $param)*) -> $out;
			)*

			/// Transform this into an `IoDelegate`, automatically wrapping
			/// the parameters.
			fn to_delegate(self) -> ::jsonrpc_core::IoDelegate<Self> {
				let mut del = ::jsonrpc_core::IoDelegate::new(self.into());
				$(
					del.add_method($rpc_name, move |base, params| {
						($name::$method as fn(&_ $(, $param)*) -> $out).wrap_rpc(base, params)
					});
				)*
				del
			}
		}
	}
}

/// A wrapper type without an implementation of `Deserialize`
/// which allows a special implementation of `Wrap` for functions
/// that take a trailing default parameter.
pub struct Trailing<T: Default + Deserialize>(pub T);

/// Wrapper trait for synchronous RPC functions.
pub trait Wrap<B: Send + Sync + 'static> {
	fn wrap_rpc(&self, base: &B, params: Params) -> Result<Value, Error>;
}

// special impl for no parameters.
impl<B, OUT> Wrap<B> for fn(&B) -> Result<OUT, Error>
	where B: Send + Sync + 'static, OUT: Serialize
{
	fn wrap_rpc(&self, base: &B, params: Params) -> Result<Value, Error> {
		::v1::helpers::params::expect_no_params(params)
			.and_then(|()| (self)(base))
			.map(to_value)
	}
}

// creates a wrapper implementation which deserializes the parameters,
// calls the function with concrete type, and serializes the output.
macro_rules! wrap {
	($($x: ident),+) => {
		impl <
			BASE: Send + Sync + 'static,
			OUT: Serialize,
			$($x: Deserialize,)+
		> Wrap<BASE> for fn(&BASE, $($x,)+) -> Result<OUT, Error> {
			fn wrap_rpc(&self, base: &BASE, params: Params) -> Result<Value, Error> {
				from_params::<($($x,)+)>(params).and_then(|($($x,)+)| {
					(self)(base, $($x,)+)
				}).map(to_value)
			}
		}
	}
}

// special impl for no parameters other than block parameter.
impl<B, OUT, T> Wrap<B> for fn(&B, Trailing<T>) -> Result<OUT, Error>
	where B: Send + Sync + 'static, OUT: Serialize, T: Default + Deserialize
{
	fn wrap_rpc(&self, base: &B, params: Params) -> Result<Value, Error> {
		let len = match params {
			Params::Array(ref v) => v.len(),
			Params::None => 0,
			_ => return Err(errors::invalid_params("not an array", "")),
		};

		let (id,) = match len {
			0 => (T::default(),),
			1 => try!(from_params::<(T,)>(params)),
			_ => return Err(Error::invalid_params()),
		};

		(self)(base, Trailing(id)).map(to_value)
	}
}

// similar to `wrap!`, but handles a single default trailing parameter
// accepts an additional argument indicating the number of non-trailing parameters.
macro_rules! wrap_with_trailing {
	($num: expr, $($x: ident),+) => {
		impl <
			BASE: Send + Sync + 'static,
			OUT: Serialize,
			$($x: Deserialize,)+
			TRAILING: Default + Deserialize,
		> Wrap<BASE> for fn(&BASE, $($x,)+ Trailing<TRAILING>) -> Result<OUT, Error> {
			fn wrap_rpc(&self, base: &BASE, params: Params) -> Result<Value, Error> {
				let len = match params {
					Params::Array(ref v) => v.len(),
					Params::None => 0,
					_ => return Err(errors::invalid_params("not an array", "")),
				};

				let params = match len - $num {
					0 => from_params::<($($x,)+)>(params)
						.map(|($($x,)+)| ($($x,)+ TRAILING::default())),
					1 => from_params::<($($x,)+ TRAILING)>(params)
						.map(|($($x,)+ id)| ($($x,)+ id)),
					_ => Err(Error::invalid_params()),
				};

				let ($($x,)+ id) = try!(params);
				(self)(base, $($x,)+ Trailing(id)).map(to_value)
			}
		}
	}
}

wrap!(A, B, C, D, E);
wrap!(A, B, C, D);
wrap!(A, B, C);
wrap!(A, B);
wrap!(A);

wrap_with_trailing!(5, A, B, C, D, E);
wrap_with_trailing!(4, A, B, C, D);
wrap_with_trailing!(3, A, B, C);
wrap_with_trailing!(2, A, B);
wrap_with_trailing!(1, A);