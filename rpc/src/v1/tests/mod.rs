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
	(iter $file:expr) => {{
		const RAW_DATA: &'static [u8] =
			include_bytes!(concat!("../../../../ethcore/res/ethereum/tests/", $file, ".json"));
		::ethjson::blockchain::Test::load(RAW_DATA).unwrap().into_iter()
	}};

	($file:expr) => {{
		extract_chain!(iter $file).filter(|&(_, ref t)| t.network == ForkSpec::Frontier).next().unwrap().1
	}};
}

macro_rules! register_test {
	($name:ident, $cb:expr, $file:expr) => {
		#[test]
		fn $name() {
			for (name, chain) in extract_chain!(iter $file).filter(|&(_, ref t)| t.network == ForkSpec::Frontier) {
				$cb(name, chain);
			}
		}
	};
}

#[cfg(test)]
mod mocked;
#[cfg(test)]
mod eth;
