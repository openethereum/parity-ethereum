// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt;
use std::marker::PhantomData;

use ethereum_types::{Address, U256};
use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use types::transaction::SignedTransaction;

/// Filtering options for the pending transactions
/// May be used for filtering transactions based on gas, gas price, value and/or nonce.
// NOTE: the fields are only `pub` because they are needed for tests
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FilterOptions {
	/// Filter based on the `sender` of the transaction.
	pub from: FilterOperator<Address>,
	/// Filter based on `receiver` of the transaction.
	pub to: FilterOperator<Option<Address>>,
	/// Filter based on `gas` of the transaction.
	pub gas: FilterOperator<U256>,
	/// Filter based on `gas price` of the transaction.
	pub gas_price: FilterOperator<U256>,
	/// Filter based on `value` of the transaction.
	pub value: FilterOperator<U256>,
	/// Filter based on `nonce` of the transaction.
	pub nonce: FilterOperator<U256>,
}

impl FilterOptions {
	fn sender_matcher(filter: &FilterOperator<Address>, candidate: &Address) -> bool {
		match filter {
			FilterOperator::Eq(address) => candidate == address,
			FilterOperator::Any => true,
			// Handled during deserialization
			_ => unreachable!(),
		}
	}

	fn receiver_matcher(filter: &FilterOperator<Option<Address>>, candidate: &Option<Address>) -> bool {
		match filter {
			FilterOperator::Eq(address) => candidate == address,
			FilterOperator::Any => true,
			// Handled during deserialization
			_ => unreachable!(),
		}
	}
	fn value_matcher(filter: &FilterOperator<U256>, tx_value: &U256) -> bool {
		match filter {
			FilterOperator::Eq(ref value) => tx_value == value,
			FilterOperator::GreaterThan(ref value) => tx_value > value,
			FilterOperator::LessThan(ref value) => tx_value < value,
			FilterOperator::Any => true,
		}
	}

	/// Determines whether a transaction passes the configured filter
	pub fn matches(&self, tx: &SignedTransaction) -> bool {
		Self::sender_matcher(&self.from, &tx.sender()) &&
		Self::receiver_matcher(&self.to, &tx.receiver()) &&
		Self::value_matcher(&self.gas, &tx.gas) &&
		Self::value_matcher(&self.gas_price, &tx.gas_price) &&
		Self::value_matcher(&self.value, &tx.value) &&
		Self::value_matcher(&self.nonce, &tx.nonce)
	}
}

impl Default for FilterOptions {
	fn default() -> Self {
		FilterOptions {
			from: FilterOperator::Any,
			to: FilterOperator::Any,
			gas: FilterOperator::Any,
			gas_price: FilterOperator::Any,
			value: FilterOperator::Any,
			nonce: FilterOperator::Any,
		}
	}
}

/// The highly generic use of implementing Deserialize for FilterOperator
/// will result in a compiler error if the type FilterOperator::Eq(None)
/// gets returned explicitly. Therefore this Wrapper will be used for
/// deserialization, directly identifying the contract creation.
enum Wrapper<T> {
	/// FilterOperations
	O(FilterOperator<T>),
	/// Contract Creation
	CC,
}

/// Available operators for filtering options.
/// The `from` filter only accepts Any and Eq(Address)
/// The `to` filter only accepts Any, Eq(Address) and Eq(None) for contract creation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilterOperator<T> {
	/// Any (no filter)
	Any,
	/// Equal
	Eq(T),
	/// Greather than
	GreaterThan(T),
	/// Less than
	LessThan(T),
}

/// Since there are multiple operators which are not supported equally by all filters,
/// this trait will validate each of those operators. The corresponding method is called
/// inside the `Deserialize` -> `Visitor` implementation for FilterOperator. In case new
/// operators get introduced, a whitelist instead of a blacklist is used.
///
/// The `from` filter validates with `validate_from`
/// The `to` filter validates with `validate_from`
/// All other filters such as gas and price validate with `validate_value`
trait Validate<'de, T, M: MapAccess<'de>> {
	fn validate_from(&mut self) -> Result<FilterOperator<T>, M::Error>;
	fn validate_to(&mut self) -> Result<FilterOperator<Option<Address>>, M::Error>;
	fn validate_value(&mut self) -> Result<FilterOperator<T>, M::Error>;
}

impl<'de, T, M> Validate<'de, T, M> for M
where
	T: Deserialize<'de>, M: MapAccess<'de>
{
	fn validate_from(&mut self) -> Result<FilterOperator<T>, M::Error> {
		use self::Wrapper as W;
		use self::FilterOperator::*;
		let wrapper = self.next_value()?;
		match wrapper {
			W::O(val) => {
				match val {
					Any | Eq(_) => Ok(val),
					_ => {
						Err(M::Error::custom(
							"the `from` filter only supports the `eq` operator",
						))
					}
				}
			},
			W::CC => {
				Err(M::Error::custom(
					"the `from` filter only supports the `eq` operator",
				))
			}
		}
	}
	fn validate_to(&mut self) -> Result<FilterOperator<Option<Address>>, M::Error> {
		use self::Wrapper as W;
		use self::FilterOperator::*;
		let wrapper = self.next_value()?;
		match wrapper {
			W::O(val) => {
				match val {
					Any => Ok(Any),
					Eq(address) => Ok(Eq(Some(address))),
					_ => {
						Err(M::Error::custom(
							"the `to` filter only supports the `eq` or `action` operator",
						))
					}
				}
			},
			W::CC => Ok(FilterOperator::Eq(None)),
		}
	}
	fn validate_value(&mut self) -> Result<FilterOperator<T>, M::Error> {
		use self::Wrapper as W;
		let wrapper = self.next_value()?;
		match wrapper {
			W::O(val) => Ok(val),
			W::CC => {
				Err(M::Error::custom(
					"the operator `action` is only supported by the `to` filter",
				))
			}
		}
	}
}

impl<'de> Deserialize<'de> for FilterOptions {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct FilterOptionsVisitor;
		impl<'de> Visitor<'de> for FilterOptionsVisitor {
			type Value = FilterOptions;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				// "This Visitor expects to receive ..."
				formatter.write_str("a map with one valid filter such as `from`, `to`, `gas`, `gas_price`, `value` or `nonce`")
			}

			fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
			where
				M: MapAccess<'de>,
			{
				let mut filter = FilterOptions::default();
				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"from" => {
							filter.from = map.validate_from()?;
						},
						"to" => {
							// Compiler cannot infer type, so set one (nothing specific for this method)
							filter.to = Validate::<(), _>::validate_to(&mut map)?;
						},
						"gas" => {
							filter.gas = map.validate_value()?;
						},
						"gas_price" => {
							filter.gas_price = map.validate_value()?;
						},
						"value" => {
							filter.value = map.validate_value()?;
						},
						"nonce" => {
							filter.nonce = map.validate_value()?;
						},
						unknown => {
							return Err(M::Error::unknown_field(
								unknown,
								&["from", "to", "gas", "gas_price", "value", "nonce"],
							))
						}
					}
				}
				Ok(filter)
			}
		}

		impl<'de, T: Deserialize<'de>> Deserialize<'de> for Wrapper<T> {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				struct WrapperVisitor<T> {
					data: PhantomData<T>,
				};
				impl<'de, T: Deserialize<'de>> Visitor<'de> for WrapperVisitor<T> {
					type Value = Wrapper<T>;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						// "This Visitor expects to receive ..."
						formatter.write_str(
							"a map with one valid operator such as `eq`, `gt` or `lt`. \
							 The to filter can also contain `action`",
						)
					}

					fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
					where
						M: MapAccess<'de>,
					{
						use self::Wrapper as W;
						let mut counter = 0;
						let mut f_op = Wrapper::O(FilterOperator::Any);

						while let Some(key) = map.next_key::<String>()? {
							match key.as_str() {
								"eq" => f_op = W::O(FilterOperator::Eq(map.next_value()?)),
								"gt" => f_op = W::O(FilterOperator::GreaterThan(map.next_value()?)),
								"lt" => f_op = W::O(FilterOperator::LessThan(map.next_value()?)),
								"action" => {
									match map.next_value()? {
										"contract_creation" => {
											f_op = W::CC;
										},
										_ => {
											return Err(M::Error::custom(
												"`action` only supports the value `contract_creation`",
											))
										}
									}
								}
								unknown => {
									// skip mentioning `action` since it's a special/rare
									// case and might confuse the usage with other filters.
									return Err(M::Error::unknown_field(unknown, &["eq", "gt", "lt"]));
								}
							}

							counter += 1;
						}

						// Good practices ensured: only one operator per filter field is allowed.
						// In case there is more than just one operator, this method must still process
						// all of them, otherwise serde returns an error mentioning a trailing comma issue
						// (even on valid JSON), which is misleading to the user of this software.
						if counter > 1 {
							return Err(M::Error::custom(
								"only one operator per filter type allowed",
							));
						}

						Ok(f_op)
					}
				}

				deserializer.deserialize_map(WrapperVisitor { data: PhantomData })
			}
		}

		deserializer.deserialize_map(FilterOptionsVisitor)
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::{Address, U256};
	use serde_json;
	use super::*;
	use std::str::FromStr;

	#[test]
	fn valid_defaults() {
		let default = FilterOptions::default();
		assert_eq!(default.from, FilterOperator::Any);
		assert_eq!(default.to, FilterOperator::Any);
		assert_eq!(default.gas, FilterOperator::Any);
		assert_eq!(default.gas_price, FilterOperator::Any);
		assert_eq!(default.value, FilterOperator::Any);
		assert_eq!(default.nonce, FilterOperator::Any);

		let json = r#"{}"#;
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, default);
	}

	#[test]
	fn valid_full_deserialization() {
		let json = r#"
			{
				"from": {
					"eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
				},
				"to": {
					"eq": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
				},
				"gas": {
					"eq": "0x493e0"
				},
				"gas_price": {
					"eq": "0x12a05f200"
				},
				"value": {
					"eq": "0x0"
				},
				"nonce": {
					"eq": "0x577"
				}
			}
		"#;

		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			from: FilterOperator::Eq(Address::from_str("5f3dffcf347944d3739b0805c934d86c8621997f").unwrap()),
			to: FilterOperator::Eq(Some(Address::from_str("e8b2d01ffa0a15736b2370b6e5064f9702c891b6").unwrap())),
			gas: FilterOperator::Eq(U256::from(300_000)),
			gas_price: FilterOperator::Eq(U256::from(5_000_000_000 as i64)),
			value: FilterOperator::Eq(U256::from(0)),
			nonce: FilterOperator::Eq(U256::from(1399)),
		})
	}

	#[test]
	fn invalid_full_deserialization() {
		// Invalid filter type `zyx`
		let json = r#"
			{
				"from": {
					"eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
				},
				"to": {
					"eq": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
				},
				"zyx": {
					"eq": "0x493e0"
				},
				"gas_price": {
					"eq": "0x12a05f200"
				},
				"value": {
					"eq": "0x0"
				},
				"nonce": {
					"eq": "0x577"
				}
			}
		"#;

		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err())
	}

	#[test]
	fn valid_from_operators() {
		// Only one valid operator for from
		let json = r#"
			{
				"from": {
					"eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			from: FilterOperator::Eq(Address::from_str("5f3dffcf347944d3739b0805c934d86c8621997f").unwrap()),
			..default
		});
	}

	#[test]
	fn invalid_from_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"from": {
					"eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f",
					"lt": "0x407d73d8a49eeb85d32cf465507dd71d507100c1"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Gt
		let json = r#"
			{
				"from": {
					"gt": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Lt
		let json = r#"
			{
				"from": {
					"lt": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action
		let json = r#"
			{
				"from": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"from": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}

	#[test]
	fn valid_to_operators() {
		// Only two valid operator for to
		// Eq
		let json = r#"
			{
				"to": {
					"eq": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			to: FilterOperator::Eq(Some(Address::from_str("e8b2d01ffa0a15736b2370b6e5064f9702c891b6").unwrap())),
			..default.clone()
		});

		// Action
		let json = r#"
			{
				"to": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			to: FilterOperator::Eq(None),
			..default
		});
	}

	#[test]
	fn invalid_to_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"to": {
					"eq": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6",
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Gt
		let json = r#"
			{
				"to": {
					"gt": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Lt
		let json = r#"
			{
				"to": {
					"lt": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action (invalid value, must be "contract_creation")
		let json = r#"
			{
				"to": {
					"action": "some_invalid_value"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"to": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}

	#[test]
	fn valid_gas_operators() {
		// Eq
		let json = r#"
			{
				"gas": {
					"eq": "0x493e0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas: FilterOperator::Eq(U256::from(300_000)),
			..default.clone()
		});

		// Gt
		let json = r#"
			{
				"gas": {
					"gt": "0x493e0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas: FilterOperator::GreaterThan(U256::from(300_000)),
			..default.clone()
		});

		// Lt
		let json = r#"
			{
				"gas": {
					"lt": "0x493e0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas: FilterOperator::LessThan(U256::from(300_000)),
			..default
		});
	}

	#[test]
	fn invalid_gas_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"gas": {
					"eq": "0x493e0",
					"lt": "0x493e0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action
		let json = r#"
			{
				"gas": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"gas": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}

	#[test]
	fn valid_gas_price_operators() {
		// Eq
		let json = r#"
			{
				"gas_price": {
					"eq": "0x12a05f200"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas_price: FilterOperator::Eq(U256::from(5_000_000_000 as i64)),
			..default.clone()
		});

		// Gt
		let json = r#"
			{
				"gas_price": {
					"gt": "0x12a05f200"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas_price: FilterOperator::GreaterThan(U256::from(5_000_000_000 as i64)),
			..default.clone()
		});

		// Lt
		let json = r#"
			{
				"gas_price": {
					"lt": "0x12a05f200"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			gas_price: FilterOperator::LessThan(U256::from(5_000_000_000 as i64)),
			..default
		});
	}

	#[test]
	fn invalid_gas_price_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"gas_price": {
					"eq": "0x12a05f200",
					"lt": "0x12a05f200"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action
		let json = r#"
			{
				"gas_price": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"gas_price": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}

	#[test]
	fn valid_value_operators() {
		// Eq
		let json = r#"
			{
				"value": {
					"eq": "0x0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			value: FilterOperator::Eq(U256::from(0)),
			..default.clone()
		});

		// Gt
		let json = r#"
			{
				"value": {
					"gt": "0x0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			value: FilterOperator::GreaterThan(U256::from(0)),
			..default.clone()
		});

		// Lt
		let json = r#"
			{
				"value": {
					"lt": "0x0"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			value: FilterOperator::LessThan(U256::from(0)),
			..default
		});
	}

	#[test]
	fn invalid_value_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"value": {
					"eq": "0x0",
					"lt": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action
		let json = r#"
			{
				"value": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"value": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}

	#[test]
	fn valid_nonce_operators() {
		// Eq
		let json = r#"
			{
				"nonce": {
					"eq": "0x577"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			nonce: FilterOperator::Eq(U256::from(1399)),
			..default.clone()
		});

		// Gt
		let json = r#"
			{
				"nonce": {
					"gt": "0x577"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			nonce: FilterOperator::GreaterThan(U256::from(1399)),
			..default.clone()
		});

		// Lt
		let json = r#"
			{
				"nonce": {
					"lt": "0x577"
				}
			}
		"#;
		let default = FilterOptions::default();
		let res = serde_json::from_str::<FilterOptions>(json).unwrap();
		assert_eq!(res, FilterOptions {
			nonce: FilterOperator::LessThan(U256::from(1399)),
			..default
		});
	}

	#[test]
	fn invalid_nonce_operators() {
		// Multiple operators are invalid
		let json = r#"
			{
				"nonce": {
					"eq": "0x577",
					"lt": "0x577"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Action
		let json = r#"
			{
				"nonce": {
					"action": "contract_creation"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());

		// Unknown operator
		let json = r#"
			{
				"nonce": {
					"abc": "0x0"
				}
			}
		"#;
		let res = serde_json::from_str::<FilterOptions>(json);
		assert!(res.is_err());
	}
}
