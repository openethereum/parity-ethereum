use ethkey::Address;
use dir::KeyDirectory;
use Error;

pub fn import_accounts(src: &KeyDirectory, dst: &KeyDirectory) -> Result<Vec<Address>, Error> {
	let accounts = try!(src.load());
	accounts.into_iter().map(|a| {
		let address = a.address.clone();
		try!(dst.insert(a));
		Ok(address)
	}).collect()
}
