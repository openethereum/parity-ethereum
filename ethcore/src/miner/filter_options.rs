use ethereum_types::{Address, U256};
use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

/// This structure provides filtering options for the pending transactions RPC API call
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FilterOptions {
    // Contains the operator to filter the sender value of the transaction
    pub sender: FilterOperator<Address>,
    // Contains the operator to filter the receiver value of the transaction
    pub receiver: FilterOperator<Address>,
    // Contains the operator to filter the gas value of the transaction
    pub gas: FilterOperator<U256>,
    // Contains the operator to filter the gas price value of the transaction
    pub gas_price: FilterOperator<U256>,
    // Contains the operator to filter the transaction value
    pub value: FilterOperator<U256>,
    // Contains the operator to filter the nonce value of the transaction
    pub nonce: FilterOperator<U256>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        FilterOptions {
            sender: FilterOperator::Any,
            receiver: FilterOperator::Any,
            gas: FilterOperator::Any,
            gas_price: FilterOperator::Any,
            value: FilterOperator::Any,
            nonce: FilterOperator::Any,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilterOperator<T> {
    Any,
    Eq(T),
    GreaterThan(T),
    LessThan(T),
    ContractCreation, // only used for `receiver`
}

/// Since there are multiple operators which are not supported equally by all filters,
/// this trait will validate each of those operators. The corresponding method is called
/// inside the `Deserialize` -> `Visitor` implementation for FilterOperator. In case new
/// operators get introduced, a whitelist instead of a blacklist is used.
///
/// The `sender` filter validates with `validate_sender`
/// The `receiver` filter validates with `validate_receiver`
/// All other filters such as gas and price validate with `validate_value`
trait Validate<'de, T, M: MapAccess<'de>> {
    fn validate_sender(&mut self) -> Result<FilterOperator<T>, M::Error>;
    fn validate_receiver(&mut self) -> Result<FilterOperator<T>, M::Error>;
    fn validate_value(&mut self) -> Result<FilterOperator<T>, M::Error>;
}

impl<'de, T, M> Validate<'de, T, M> for M 
    where T: Deserialize<'de>, M: MapAccess<'de>
{
    fn validate_sender(&mut self) -> Result<FilterOperator<T>, M::Error> {
        use self::FilterOperator::*;
        let val = self.next_value()?;
        match val {
            Any | Eq(_) => Ok(val),
            _ => {
                Err(M::Error::custom(
                    "the sender filter only supports the `eq` operator",
                ))
            }
        }
    }
    fn validate_receiver(&mut self) -> Result<FilterOperator<T>, M::Error> {
        use self::FilterOperator::*;
        let val = self.next_value()?;
        match val {
            Any | Eq(_) | ContractCreation => Ok(val),
            _ => {
                Err(M::Error::custom(
                    "the sender filter only supports the `eq` and `action` operators",
                ))
            }
        }
    }
    fn validate_value(&mut self) -> Result<FilterOperator<T>, M::Error> {
        use self::FilterOperator::*;
        let val = self.next_value()?;
        match val {
            Any | Eq(_) | GreaterThan(_) | LessThan(_) => Ok(val),
            ContractCreation => {
                Err(M::Error::custom(
                    "the operator `action` is only supported by the receiver filter",
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
                formatter.write_str("a map with one valid filter such as `sender`, `receiver`, `gas`, `gas_price`, `value` or `nonce`")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut filter = FilterOptions::default();
                while let Some(key) = map.next_key()? {
                    match key {
                        "sender" => {
                            filter.sender = map.validate_sender()?;
                        },
                        "receiver" => {
                            filter.receiver = map.validate_receiver()?;
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
                                &["sender", "receiver", "gas", "gas_price", "value", "nonce"],
                            ))
                        }
                    }
                }

                Ok(filter)
            }
        }

        impl<'de, T: Deserialize<'de>> Deserialize<'de> for FilterOperator<T> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FilterOperatorVisitor<T> {
                    data: PhantomData<T>,
                };
                impl<'de, T: Deserialize<'de>> Visitor<'de> for FilterOperatorVisitor<T> {
                    type Value = FilterOperator<T>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        // "This Visitor expects to receive ..."
                        formatter.write_str(
                            "a map with one valid operator such as `eq`, `gt` or `lt`. \
                             The receiver filter can also contain `action`",
                        )
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                    where
                        M: MapAccess<'de>,
                    {
                        let mut counter = 0;
                        let mut f_op = FilterOperator::Any;

                        while let Some(key) = map.next_key()? {
                            match key {
                                "eq" => f_op = FilterOperator::Eq(map.next_value()?),
                                "gt" => f_op = FilterOperator::GreaterThan(map.next_value()?),
                                "lt" => f_op = FilterOperator::LessThan(map.next_value()?),
                                "action" => {
                                    match map.next_value()? {
                                        "contract_creation" => {
                                            f_op = FilterOperator::ContractCreation;
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

                deserializer.deserialize_map(FilterOperatorVisitor { data: PhantomData })
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
        assert_eq!(default.sender, FilterOperator::Any);
        assert_eq!(default.receiver, FilterOperator::Any);
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
                "sender": {
                    "eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
                },
                "receiver": {
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
            sender: FilterOperator::Eq(Address::from_str("5f3dffcf347944d3739b0805c934d86c8621997f").unwrap()),
            receiver: FilterOperator::Eq(Address::from_str("e8b2d01ffa0a15736b2370b6e5064f9702c891b6").unwrap()),
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
                "sender": {
                    "eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
                },
                "receiver": {
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
    fn valid_sender_operators() {
        // Only one valid operator for sender
        let json = r#"
            {
                "sender": {
                    "eq": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
                }
            }
        "#;
        let default = FilterOptions::default();
        let res = serde_json::from_str::<FilterOptions>(json).unwrap();
        assert_eq!(res, FilterOptions {
            sender: FilterOperator::Eq(Address::from_str("5f3dffcf347944d3739b0805c934d86c8621997f").unwrap()),
            ..default
        });
    }

    #[test]
    fn invalid_sender_operators() {
        // Multiple operators are invalid
        let json = r#"
            {
                "sender": {
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
                "sender": {
                    "gt": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Lt
        let json = r#"
            {
                "sender": {
                    "lt": "0x5f3dffcf347944d3739b0805c934d86c8621997f"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Action
        let json = r#"
            {
                "sender": {
                    "action": "contract_creation"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Unknown operator
        let json = r#"
            {
                "sender": {
                    "abc": "0x0"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());
    }

    #[test]
    fn valid_receiver_operators() {
        // Only two valid operator for receiver
        // Eq
        let json = r#"
            {
                "receiver": {
                    "eq": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
                }
            }
        "#;
        let default = FilterOptions::default();
        let res = serde_json::from_str::<FilterOptions>(json).unwrap();
        assert_eq!(res, FilterOptions {
            receiver: FilterOperator::Eq(Address::from_str("e8b2d01ffa0a15736b2370b6e5064f9702c891b6").unwrap()),
            ..default.clone()
        });

        // Action
        let json = r#"
            {
                "receiver": {
                    "action": "contract_creation"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json).unwrap();
        assert_eq!(res, FilterOptions {
            receiver: FilterOperator::ContractCreation,
            ..default
        });
    }

    #[test]
    fn invalid_receiver_operators() {
        // Multiple operators are invalid
        let json = r#"
            {
                "receiver": {
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
                "receiver": {
                    "gt": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Lt
        let json = r#"
            {
                "receiver": {
                    "lt": "0xe8b2d01ffa0a15736b2370b6e5064f9702c891b6"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Action (invalid value, must be "contract_creation")
        let json = r#"
            {
                "receiver": {
                    "action": "some_invalid_value"
                }
            }
        "#;
        let res = serde_json::from_str::<FilterOptions>(json);
        assert!(res.is_err());

        // Unknown operator
        let json = r#"
            {
                "receiver": {
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