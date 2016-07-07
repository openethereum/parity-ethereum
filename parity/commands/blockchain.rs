
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum DataFormat {
	Hex,
	Binary,
}

impl FromStr for DataFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"binary" | "bin" => Ok(DataFormat::Binary),
			"hex" => Ok(DataFormat::Hex),
			x => Err(format!("Invalid format: {}", x))
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;
	use super::DataFormat;

	#[test]
	fn test_data_format_parsing() {
		assert_eq!(DataFormat::from_str("binary").unwrap(), DataFormat::Binary);
		assert_eq!(DataFormat::from_str("bin").unwrap(), DataFormat::Binary);
		assert_eq!(DataFormat::from_str("hex").unwrap(), DataFormat::Hex);
	}
}
