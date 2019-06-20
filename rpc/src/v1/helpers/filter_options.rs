use ethereum_types::{Address, U256};
use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct FilterOptions {
    sender: FilterOperator<Address>,
    receiver: FilterOperator<Option<Address>>,
    gas: FilterOperator<U256>,
    gas_price: FilterOperator<U256>,
    value: FilterOperator<U256>,
    nonce: FilterOperator<U256>,
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

#[derive(Debug, Clone)]
pub enum FilterOperator<T> {
    Any,
    Eq(T),
    GreaterThan(T),
    LessThan(T),
    ContractCreation, // only used for `receiver`
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
                            filter.sender = map.next_value()?;
                        },
                        //"receiver" => match map.next_value()? {
                        "receiver" => {
                            filter.receiver = FilterOperator::ContractCreation;
                            /*
                            FilterOperator::Eq(inner_v) => {
                                filter.receiver = FilterOperator::Eq(Some(inner_v));
                            }
                            FilterOperator::ContractCreation(inner_v) => match inner_v.as_ptr() {
                                "contract_creation" => filter.receiver = FilterOperator::ContractCreation(None),
                                _ => {
                                    return Err(M::Error::custom(
                                        "`action` only supports the value `contract_creation`",
                                    ))
                                }
                            },
                            _ => {
                                return Err(M::Error::custom(
                                    "the receiver filter only supports the `eq` or `action` operator",
                                ))
                            }
                            */
                        },
                        "gas" => {
                            filter.gas = map.next_value()?;
                        },
                        "gas_price" => {
                            filter.gas_price = map.next_value()?;
                        },
                        "value" => {
                            filter.value = map.next_value()?;
                        },
                        "nonce" => {
                            filter.nonce = map.next_value()?;
                        },
                        uf @ _ => {
                            return Err(M::Error::unknown_field(
                                uf,
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
                                uf @ _ => {
                                    // skip mentioning `action` since it's a special/rare
                                    // case and might confuse the usage with other filters.
                                    return Err(M::Error::unknown_field(uf, &["eq", "gt", "lt"]));
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
