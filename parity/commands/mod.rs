
mod account;
mod presale;

pub enum Cmd {
	Run,
	Account(account::AccountCmd),
	PresaleWalletImport(presale::PresaleCmd),
	Blockchain(BlockchainCmd),
	SignerToken,
}

pub enum BlockchainCmd {
	Import,
	Export,
}

pub fn execute(command: Cmd) -> Result<(), String> {
	match command {
		Cmd::Run => {
			unimplemented!();
		},
		Cmd::Account(account_cmd) => account::execute(account_cmd),
		Cmd::PresaleWalletImport(presale_cmd) => presale::execute(presale_cmd),
		Cmd::Blockchain(_blockchain_cmd) => {
			unimplemented!();
		},
		Cmd::SignerToken => {
			unimplemented!();
		},
	}
}
