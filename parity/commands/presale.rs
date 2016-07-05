use ethcore::ethstore::{PresaleWallet, EthStore};
use ethcore::ethstore::dir::DiskDirectory;
use ethcore::account_provider::AccountProvider;

pub struct PresaleCmd {
	iterations: u32,
	path: String,
	wallet_path: String,
	password: String,
}

pub fn execute(cmd: PresaleCmd) -> Result<(), String> {
	let dir = Box::new(DiskDirectory::create(cmd.path).unwrap());
	let secret_store = Box::new(EthStore::open_with_iterations(dir, cmd.iterations).unwrap());
	let acc_provider = AccountProvider::new(secret_store);
	let wallet = try!(PresaleWallet::open(cmd.wallet_path).map_err(|_| "Unable to open presale wallet."));
	let kp = try!(wallet.decrypt(&cmd.password).map_err(|_| "Invalid password."));
	let _address = acc_provider.insert_account(kp.secret().clone(), &cmd.password).unwrap();
	Ok(())
}
