// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{fs, io, sync::Arc, time::Instant};

use ansi_term::Colour;
use bytes::ToPretty;
use cache::CacheConfig;
use db;
use dir::Directories;
use ethcore::{
    client::{
        Balance, BlockChainClient, BlockChainReset, BlockId, DatabaseCompactionProfile,
        ImportExportBlocks, Mode, Nonce, VMType,
    },
    miner::Miner,
    verification::queue::VerifierSettings,
};
use ethcore_service::ClientService;
use ethereum_types::{Address, H256, U256};
use hash::{keccak, KECCAK_NULL_RLP};
use helpers::{execute_upgrades, to_client_config};
use informant::{FullNodeInformantData, Informant, MillisecondDuration};
use params::{fatdb_switch_to_bool, tracing_switch_to_bool, Pruning, SpecType, Switch};
use types::data_format::DataFormat;
use user_defaults::UserDefaults;

#[derive(Debug, PartialEq)]
pub enum BlockchainCmd {
    Kill(KillBlockchain),
    Import(ImportBlockchain),
    Export(ExportBlockchain),
    ExportState(ExportState),
    Reset(ResetBlockchain),
}

#[derive(Debug, PartialEq)]
pub struct ResetBlockchain {
    pub dirs: Directories,
    pub spec: SpecType,
    pub pruning: Pruning,
    pub pruning_history: u64,
    pub pruning_memory: usize,
    pub tracing: Switch,
    pub fat_db: Switch,
    pub compaction: DatabaseCompactionProfile,
    pub cache_config: CacheConfig,
    pub num: u32,
}

#[derive(Debug, PartialEq)]
pub struct KillBlockchain {
    pub spec: SpecType,
    pub dirs: Directories,
    pub pruning: Pruning,
}

#[derive(Debug, PartialEq)]
pub struct ImportBlockchain {
    pub spec: SpecType,
    pub cache_config: CacheConfig,
    pub dirs: Directories,
    pub file_path: Option<String>,
    pub format: Option<DataFormat>,
    pub pruning: Pruning,
    pub pruning_history: u64,
    pub pruning_memory: usize,
    pub compaction: DatabaseCompactionProfile,
    pub tracing: Switch,
    pub fat_db: Switch,
    pub vm_type: VMType,
    pub check_seal: bool,
    pub with_color: bool,
    pub verifier_settings: VerifierSettings,
    pub max_round_blocks_to_import: usize,
}

#[derive(Debug, PartialEq)]
pub struct ExportBlockchain {
    pub spec: SpecType,
    pub cache_config: CacheConfig,
    pub dirs: Directories,
    pub file_path: Option<String>,
    pub format: Option<DataFormat>,
    pub pruning: Pruning,
    pub pruning_history: u64,
    pub pruning_memory: usize,
    pub compaction: DatabaseCompactionProfile,
    pub fat_db: Switch,
    pub tracing: Switch,
    pub from_block: BlockId,
    pub to_block: BlockId,
    pub check_seal: bool,
    pub max_round_blocks_to_import: usize,
}

#[derive(Debug, PartialEq)]
pub struct ExportState {
    pub spec: SpecType,
    pub cache_config: CacheConfig,
    pub dirs: Directories,
    pub file_path: Option<String>,
    pub format: Option<DataFormat>,
    pub pruning: Pruning,
    pub pruning_history: u64,
    pub pruning_memory: usize,
    pub compaction: DatabaseCompactionProfile,
    pub fat_db: Switch,
    pub tracing: Switch,
    pub at: BlockId,
    pub storage: bool,
    pub code: bool,
    pub min_balance: Option<U256>,
    pub max_balance: Option<U256>,
    pub max_round_blocks_to_import: usize,
}

pub fn execute(cmd: BlockchainCmd) -> Result<(), String> {
    match cmd {
        BlockchainCmd::Kill(kill_cmd) => kill_db(kill_cmd),
        BlockchainCmd::Import(import_cmd) => execute_import(import_cmd),
        BlockchainCmd::Export(export_cmd) => execute_export(export_cmd),
        BlockchainCmd::ExportState(export_cmd) => execute_export_state(export_cmd),
        BlockchainCmd::Reset(reset_cmd) => execute_reset(reset_cmd),
    }
}

fn execute_import(cmd: ImportBlockchain) -> Result<(), String> {
    let timer = Instant::now();

    // load spec file
    let spec = cmd.spec.spec(&cmd.dirs.cache)?;

    // load genesis hash
    let genesis_hash = spec.genesis_header().hash();

    // database paths
    let db_dirs = cmd.dirs.database(genesis_hash, None, spec.data_dir.clone());

    // user defaults path
    let user_defaults_path = db_dirs.user_defaults_path();

    // load user defaults
    let mut user_defaults = UserDefaults::load(&user_defaults_path)?;

    // select pruning algorithm
    let algorithm = cmd.pruning.to_algorithm(&user_defaults);

    // check if tracing is on
    let tracing = tracing_switch_to_bool(cmd.tracing, &user_defaults)?;

    // check if fatdb is on
    let fat_db = fatdb_switch_to_bool(cmd.fat_db, &user_defaults, algorithm)?;

    // prepare client and snapshot paths.
    let client_path = db_dirs.client_path(algorithm);
    let snapshot_path = db_dirs.snapshot_path();

    // execute upgrades
    execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, &cmd.compaction)?;

    // create dirs used by parity
    cmd.dirs.create_dirs(false, false)?;

    // prepare client config
    let mut client_config = to_client_config(
        &cmd.cache_config,
        spec.name.to_lowercase(),
        Mode::Active,
        tracing,
        fat_db,
        cmd.compaction,
        cmd.vm_type,
        "".into(),
        algorithm,
        cmd.pruning_history,
        cmd.pruning_memory,
        cmd.check_seal,
        12,
    );

    client_config.queue.verifier_settings = cmd.verifier_settings;

    let restoration_db_handler = db::restoration_db_handler(&client_path, &client_config);
    let client_db = restoration_db_handler
        .open(&client_path)
        .map_err(|e| format!("Failed to open database {:?}", e))?;

    // build client
    let service = ClientService::start(
        client_config,
        &spec,
        client_db,
        &snapshot_path,
        restoration_db_handler,
        &cmd.dirs.ipc_path(),
        // TODO [ToDr] don't use test miner here
        // (actually don't require miner at all)
        Arc::new(Miner::new_for_tests(&spec, None)),
    )
    .map_err(|e| format!("Client service error: {:?}", e))?;

    // free up the spec in memory.
    drop(spec);

    let client = service.client();

    let instream: Box<dyn io::Read> = match cmd.file_path {
        Some(f) => {
            Box::new(fs::File::open(&f).map_err(|_| format!("Cannot open given file: {}", f))?)
        }
        None => Box::new(io::stdin()),
    };

    let informant = Arc::new(Informant::new(
        FullNodeInformantData {
            client: client.clone(),
            sync: None,
            net: None,
        },
        None,
        None,
        cmd.with_color,
    ));

    service
        .register_io_handler(informant)
        .map_err(|_| "Unable to register informant handler".to_owned())?;

    client.import_blocks(instream, cmd.format)?;

    // save user defaults
    user_defaults.pruning = algorithm;
    user_defaults.tracing = tracing;
    user_defaults.fat_db = fat_db;
    user_defaults.save(&user_defaults_path)?;

    let report = client.report();

    let ms = timer.elapsed().as_milliseconds();
    info!("Import completed in {} seconds, {} blocks, {} blk/s, {} transactions, {} tx/s, {} Mgas, {} Mgas/s",
		ms / 1000,
		report.blocks_imported,
		(report.blocks_imported * 1000) as u64 / ms,
		report.transactions_applied,
		(report.transactions_applied * 1000) as u64 / ms,
		report.gas_processed / 1_000_000,
		(report.gas_processed / (ms * 1000)).low_u64(),
	);
    Ok(())
}

fn start_client(
    dirs: Directories,
    spec: SpecType,
    pruning: Pruning,
    pruning_history: u64,
    pruning_memory: usize,
    tracing: Switch,
    fat_db: Switch,
    compaction: DatabaseCompactionProfile,
    cache_config: CacheConfig,
    require_fat_db: bool,
    max_round_blocks_to_import: usize,
) -> Result<ClientService, String> {
    // load spec file
    let spec = spec.spec(&dirs.cache)?;

    // load genesis hash
    let genesis_hash = spec.genesis_header().hash();

    // database paths
    let db_dirs = dirs.database(genesis_hash, None, spec.data_dir.clone());

    // user defaults path
    let user_defaults_path = db_dirs.user_defaults_path();

    // load user defaults
    let user_defaults = UserDefaults::load(&user_defaults_path)?;

    // select pruning algorithm
    let algorithm = pruning.to_algorithm(&user_defaults);

    // check if tracing is on
    let tracing = tracing_switch_to_bool(tracing, &user_defaults)?;

    // check if fatdb is on
    let fat_db = fatdb_switch_to_bool(fat_db, &user_defaults, algorithm)?;
    if !fat_db && require_fat_db {
        return Err("This command requires OpenEthereum to be synced with --fat-db on.".to_owned());
    }

    // prepare client and snapshot paths.
    let client_path = db_dirs.client_path(algorithm);
    let snapshot_path = db_dirs.snapshot_path();

    // execute upgrades
    execute_upgrades(&dirs.base, &db_dirs, algorithm, &compaction)?;

    // create dirs used by OpenEthereum.
    dirs.create_dirs(false, false)?;

    // prepare client config
    let client_config = to_client_config(
        &cache_config,
        spec.name.to_lowercase(),
        Mode::Active,
        tracing,
        fat_db,
        compaction,
        VMType::default(),
        "".into(),
        algorithm,
        pruning_history,
        pruning_memory,
        true,
        max_round_blocks_to_import,
    );

    let restoration_db_handler = db::restoration_db_handler(&client_path, &client_config);
    let client_db = restoration_db_handler
        .open(&client_path)
        .map_err(|e| format!("Failed to open database {:?}", e))?;

    let service = ClientService::start(
        client_config,
        &spec,
        client_db,
        &snapshot_path,
        restoration_db_handler,
        &dirs.ipc_path(),
        // It's fine to use test version here,
        // since we don't care about miner parameters at all
        Arc::new(Miner::new_for_tests(&spec, None)),
    )
    .map_err(|e| format!("Client service error: {:?}", e))?;

    drop(spec);
    Ok(service)
}

fn execute_export(cmd: ExportBlockchain) -> Result<(), String> {
    let service = start_client(
        cmd.dirs,
        cmd.spec,
        cmd.pruning,
        cmd.pruning_history,
        cmd.pruning_memory,
        cmd.tracing,
        cmd.fat_db,
        cmd.compaction,
        cmd.cache_config,
        false,
        cmd.max_round_blocks_to_import,
    )?;
    let client = service.client();

    let out: Box<dyn io::Write> = match cmd.file_path {
        Some(f) => Box::new(
            fs::File::create(&f).map_err(|_| format!("Cannot write to file given: {}", f))?,
        ),
        None => Box::new(io::stdout()),
    };

    client.export_blocks(out, cmd.from_block, cmd.to_block, cmd.format)?;

    info!("Export completed.");
    Ok(())
}

fn execute_export_state(cmd: ExportState) -> Result<(), String> {
    let service = start_client(
        cmd.dirs,
        cmd.spec,
        cmd.pruning,
        cmd.pruning_history,
        cmd.pruning_memory,
        cmd.tracing,
        cmd.fat_db,
        cmd.compaction,
        cmd.cache_config,
        true,
        cmd.max_round_blocks_to_import,
    )?;

    let client = service.client();

    let mut out: Box<dyn io::Write> = match cmd.file_path {
        Some(f) => Box::new(
            fs::File::create(&f).map_err(|_| format!("Cannot write to file given: {}", f))?,
        ),
        None => Box::new(io::stdout()),
    };

    let mut last: Option<Address> = None;
    let at = cmd.at;
    let mut i = 0usize;

    out.write_fmt(format_args!("{{ \"state\": {{",))
        .expect("Couldn't write to stream.");
    loop {
        let accounts = client
            .list_accounts(at, last.as_ref(), 1000)
            .ok_or("Specified block not found")?;
        if accounts.is_empty() {
            break;
        }

        for account in accounts.into_iter() {
            let balance = client
                .balance(&account, at.into())
                .unwrap_or_else(U256::zero);
            if cmd.min_balance.map_or(false, |m| balance < m)
                || cmd.max_balance.map_or(false, |m| balance > m)
            {
                last = Some(account);
                continue; //filtered out
            }

            if i != 0 {
                out.write(b",").expect("Write error");
            }
            out.write_fmt(format_args!(
                "\n\"0x{:x}\": {{\"balance\": \"{:x}\", \"nonce\": \"{:x}\"",
                account,
                balance,
                client.nonce(&account, at).unwrap_or_else(U256::zero)
            ))
            .expect("Write error");
            let code = client
                .code(&account, at.into())
                .unwrap_or(None)
                .unwrap_or_else(Vec::new);
            if !code.is_empty() {
                out.write_fmt(format_args!(", \"code_hash\": \"0x{:x}\"", keccak(&code)))
                    .expect("Write error");
                if cmd.code {
                    out.write_fmt(format_args!(", \"code\": \"{}\"", code.to_hex()))
                        .expect("Write error");
                }
            }
            let storage_root = client.storage_root(&account, at).unwrap_or(KECCAK_NULL_RLP);
            if storage_root != KECCAK_NULL_RLP {
                out.write_fmt(format_args!(", \"storage_root\": \"0x{:x}\"", storage_root))
                    .expect("Write error");
                if cmd.storage {
                    out.write_fmt(format_args!(", \"storage\": {{"))
                        .expect("Write error");
                    let mut last_storage: Option<H256> = None;
                    loop {
                        let keys = client
                            .list_storage(at, &account, last_storage.as_ref(), 1000)
                            .ok_or("Specified block not found")?;
                        if keys.is_empty() {
                            break;
                        }

                        for key in keys.into_iter() {
                            if last_storage.is_some() {
                                out.write(b",").expect("Write error");
                            }
                            out.write_fmt(format_args!(
                                "\n\t\"0x{:x}\": \"0x{:x}\"",
                                key,
                                client
                                    .storage_at(&account, &key, at.into())
                                    .unwrap_or_else(Default::default)
                            ))
                            .expect("Write error");
                            last_storage = Some(key);
                        }
                    }
                    out.write(b"\n}").expect("Write error");
                }
            }
            out.write(b"}").expect("Write error");
            i += 1;
            if i % 10000 == 0 {
                info!("Account #{}", i);
            }
            last = Some(account);
        }
    }
    out.write_fmt(format_args!("\n}}}}")).expect("Write error");
    info!("Export completed.");
    Ok(())
}

fn execute_reset(cmd: ResetBlockchain) -> Result<(), String> {
    let service = start_client(
        cmd.dirs,
        cmd.spec,
        cmd.pruning,
        cmd.pruning_history,
        cmd.pruning_memory,
        cmd.tracing,
        cmd.fat_db,
        cmd.compaction,
        cmd.cache_config,
        false,
        0,
    )?;

    let client = service.client();
    client.reset(cmd.num)?;
    info!("{}", Colour::Green.bold().paint("Successfully reset db!"));

    Ok(())
}

pub fn kill_db(cmd: KillBlockchain) -> Result<(), String> {
    let spec = cmd.spec.spec(&cmd.dirs.cache)?;
    let genesis_hash = spec.genesis_header().hash();
    let db_dirs = cmd.dirs.database(genesis_hash, None, spec.data_dir);
    let user_defaults_path = db_dirs.user_defaults_path();
    let mut user_defaults = UserDefaults::load(&user_defaults_path)?;
    let algorithm = cmd.pruning.to_algorithm(&user_defaults);
    let dir = db_dirs.db_path(algorithm);
    fs::remove_dir_all(&dir).map_err(|e| format!("Error removing database: {:?}", e))?;
    user_defaults.is_first_launch = true;
    user_defaults.save(&user_defaults_path)?;
    info!("Database deleted.");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::DataFormat;

    #[test]
    fn test_data_format_parsing() {
        assert_eq!(DataFormat::Binary, "binary".parse().unwrap());
        assert_eq!(DataFormat::Binary, "bin".parse().unwrap());
        assert_eq!(DataFormat::Hex, "hex".parse().unwrap());
    }
}
