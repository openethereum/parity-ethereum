// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Client tests of tracing

use ethkey::KeyPair;
use block::*;
use util::*;
use io::*;
use spec::*;
use client::*;
use tests::helpers::*;
use devtools::RandomTempPath;
use client::{BlockChainClient, Client, ClientConfig};
use util::kvdb::{Database, DatabaseConfig};
use std::sync::Arc;
use header::Header;
use miner::Miner;
use transaction::{Action, Transaction};

#[test]
fn can_trace_block_and_uncle_reward() {
	let dir = RandomTempPath::new();
    let spec = Spec::new_test_with_reward();
    let engine = &*spec.engine;
    let genesis_header = spec.genesis_header();    
    let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();	
    let last_hashes = Arc::new(vec![genesis_header.hash()]);
    let mut b = OpenBlock::new(engine, Default::default(), true, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false).unwrap();

	let kp = KeyPair::from_secret_slice(&"".sha3()).unwrap();
	let mut n = 0;
	for _ in 0..1 {
			b.push_transaction(Transaction {
				nonce: n.into(),
				gas_price: 0.into(),
				gas: 100000.into(),
				action: Action::Create,
				data: vec![],
				value: U256::zero(),
			}.sign(kp.secret(), Some(spec.network_id())), None).unwrap();
			n += 1;
	}

    let mut uncle = Header::new();
    let uncle_author: Address = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
    uncle.set_author(uncle_author);
    //b.push_uncle(uncle).unwrap();
    let b = b.close_and_lock().seal(engine, vec![]).unwrap();

    let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
	let mut client_config = ClientConfig::default();
	client_config.tracing.enabled = true;
    let client_db = Arc::new(Database::open(&db_config, dir.as_path().to_str().unwrap()).unwrap());
    let client = Client::new(
		client_config,
		&spec,
		client_db,
		Arc::new(Miner::with_spec(&spec)),
		IoChannel::disconnected(),
	).unwrap();
	
	let res = client.import_block(b.rlp_bytes());
    if res.is_err() {
		panic!("error importing block: {:#?}", res.err().unwrap());        
	}
    
	client.flush_queue();
	client.import_verified_blocks();

    let filter = TraceFilter {
			range: (BlockId::Number(1)..BlockId::Number(1)),
			from_address: vec![],
			to_address: vec![],
		};

    let traces = client.filter_traces(filter);    
    assert!(traces.is_some(), "Genesis trace should be always present.");
    //panic!("Traces size is: {:#?}", traces.unwrap().len());    
}