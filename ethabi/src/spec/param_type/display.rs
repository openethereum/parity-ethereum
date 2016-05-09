use std::fmt::{Display, Formatter, Error};
use super::{ParamType, Writer};

impl Display for ParamType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{}", Writer::write(self))
	}
}

#[cfg(test)]
mod tests {
	use spec::ParamType;

	#[test]
	fn test_param_type_display() {
		assert_eq!(format!("{}", ParamType::Address), "address".to_owned());
		assert_eq!(format!("{}", ParamType::Bytes), "bytes".to_owned());
		assert_eq!(format!("{}", ParamType::FixedBytes(32)), "bytes32".to_owned());
		assert_eq!(format!("{}", ParamType::Uint(256)), "uint256".to_owned());
		assert_eq!(format!("{}", ParamType::Int(64)), "int64".to_owned());
		assert_eq!(format!("{}", ParamType::Bool), "bool".to_owned());
		assert_eq!(format!("{}", ParamType::String), "string".to_owned());
		assert_eq!(format!("{}", ParamType::Array(Box::new(ParamType::Bool))), "bool[]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::String), 2)), "string[2]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2)), "bool[][2]".to_owned());
	}
}
