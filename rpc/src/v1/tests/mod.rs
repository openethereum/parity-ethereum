//! RPC unit test moduleS

pub mod helpers;

// extract a chain from the given JSON file,
// stored in ethcore/res/ethereum/tests/.
//
// usage:
//     `extract_chain!("Folder/File")` will load Folder/File.json and extract
//     the first block chain stored within.
//
//     `extract_chain!("Folder/File", "with_name")` will load Folder/File.json and
//     extract the chain with that name. This will panic if no chain by that name
//     is found.
macro_rules! extract_chain {
	($file:expr, $name:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../ethcore/res/ethereum/tests/", $file, ".json"));
		let mut chain = None;
		for (name, c) in ::ethjson::blockchain::Test::load(RAW_DATA).unwrap() {
			if name == $name {
				chain = Some(c);
				break;
			}
		}
		chain.unwrap()
	}};

	($file:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../ethcore/res/ethereum/tests/", $file, ".json"));

		::ethjson::blockchain::Test::load(RAW_DATA)
			.unwrap().into_iter().next().unwrap().1
	}};
}

#[cfg(test)]
mod mocked;
#[cfg(test)]
mod eth;
