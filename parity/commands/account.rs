use ethcore::ethstore::{EthStore, import_accounts};
use ethcore::ethstore::dir::DiskDirectory;
use ethcore::account_provider::AccountProvider;

pub enum AccountCmd {
	New(NewAccount),
	List(String),
	Import(ImportAccounts),
}

pub struct NewAccount {
	iterations: u32,
	path: String,
	password: String,
}

pub struct ImportAccounts {
	from: Vec<String>,
	to: String,
}

pub fn execute(cmd: AccountCmd) -> Result<(), String> {
	match cmd {
		AccountCmd::New(new_cmd) => new(new_cmd),
		AccountCmd::List(path) => list(path),
		AccountCmd::Import(import_cmd) => import(import_cmd),
	}
}

fn new(n: NewAccount) -> Result<(), String> {
	let dir = Box::new(DiskDirectory::create(n.path).unwrap());
	let secret_store = Box::new(EthStore::open_with_iterations(dir, n.iterations).unwrap());
	let acc_provider = AccountProvider::new(secret_store);
	let _new_account = acc_provider.new_account(&n.password).unwrap();
	Ok(())
}

fn list(path: String) -> Result<(), String> {
	let dir = Box::new(DiskDirectory::create(path).unwrap());
	let secret_store = Box::new(EthStore::open(dir).unwrap());
	let acc_provider = AccountProvider::new(secret_store);
	let _accounts = acc_provider.accounts();
	Ok(())
}

fn import(i: ImportAccounts) -> Result<(), String> {
	let to = DiskDirectory::create(i.to).unwrap();
	let mut imported = 0;
	for path in &i.from {
		let from = DiskDirectory::at(path);
		imported += try!(import_accounts(&from, &to).map_err(|_| "Importing accounts failed.")).len();
	}
	Ok(())
}
