// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! This module implements Ethereum Node Records as defined in EIP-778. A node record holds
/// arbitrary information about a node on the p2p network.

/// Node information is stored in key/value pairs.

/// Node records must be signed before transmitting them to another node.

/// Decoding a record doesn't check its signature.
/// When receiving records from untrusted peer, A node must verify two things:
/// - identity scheme.
/// - the signature is valid according to the declared scheme.

/// Whenever you create or modify a record, use a signing function provided by
/// the identity scheme to add the signature.

/// This file is based on geth(https://github.com/ethereum/go-ethereum/pull/15585)

use std::collections::{BTreeMap, HashMap};
use rlp::{encode, Encodable, RlpStream};
use std::net::IpAddr;
use log::error;

/// Maximum encoded size of a node record in bytes.
pub const ENR_MAX_SIZE: usize = 300;

/// A node record entry.
/// A new node record entry should implements this trait.
pub trait Entry {
    fn enr_key(&self) -> String;
}

/// Provide custom verify method for each scheme.
pub trait Scheme {
    fn verify(&self, r: &Record, sig: &[u8]) -> bool;
}

/// Holds scheme for some type.
pub struct SchemeMap<T>(HashMap<String, T>);

impl<T: Scheme> SchemeMap<T> {
    /// Create new object.
    pub fn new() -> Self {
        let map = HashMap::new();
        SchemeMap(map)
    }

    /// Top level function for verifying signature.
    pub fn verify(&self, record: &Record, sig: &Vec<u8>) -> bool {
        if let Some(scheme_id) = record.scheme() {
            if let Some(scheme) = self.0.get(&scheme_id) {
                scheme.verify(record, sig)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Add more scheme to support
    pub fn add(&mut self, scheme_id: String, scheme: T) {
        self.0.insert(scheme_id,scheme);
    }
}

/// Value type for KeyValue pairs in Record struct.
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    /// Tcp port
    Tcp(Tcp),
    /// Tcp port for IPv6
    Tcp6(Tcp6),
    /// Udp port
    Udp(Udp),
    /// Udp port for IPv6
    Udp6(Udp6),
    /// Scheme Id
    Id(Id),
    /// IPv4 or IPv6 address
    IpAddr(IpAddr),
}

impl Entry for Value {
    fn enr_key(&self) -> String {
        match self {
            Value::Tcp(tcp) => tcp.enr_key(),
            Value::Tcp6(tcp6) => tcp6.enr_key(),
            Value::Udp(udp) => udp.enr_key(),
            Value::Udp6(udp6) => udp6.enr_key(),
            Value::Id(id) => id.enr_key(),
            Value::IpAddr(ip) => ip.enr_key(),
        }
    }
}

/// A node record as defined in EIP-778.
#[derive(Default)]
pub struct Record {
    seq: u64,
    signature: Option<Vec<u8>>,
    raw: Option<Vec<u8>>, // RLP encoded record.
    pairs: BTreeMap<String, Value>, // Sorted list of key/value pairs.
}

impl Record {
    pub fn get(&self, entry: &dyn Entry) -> Option<&Value> {
        self.pairs.get(&entry.enr_key())
    }

    /// Add or update a value for a key.
    pub fn set(&mut self, value: Value) {
        self.invalidate();
        self.pairs.insert(value.enr_key(), value);
    }

    fn invalidate(&mut self) {
        if self.signature.is_some() {
            self.seq += 1;
        }

        self.signature = None;
        self.raw = None;
    }

    pub fn signature(&self) -> Option<Vec<u8>> {
        self.signature.clone()
    }

    /// Get the scheme represented by a string.
    /// Since parameter `id` of `get()` has a `Id` type,
    /// `get` can call `enr_key()`.
    /// So, we just pass it with empty string.
    pub fn scheme(&self) -> Option<String> {
        let id = Id("".to_owned());
        let value = self.get(&id);
        match value {
            Some(Value::Id(id)) => Some(id.0.clone()),
            _ => {
                error!(target: "network", "Can't get scheme for ENR");
                None
            },
        }
    }

    pub fn set_signature(&mut self, scheme: &dyn Scheme, sig: &Vec<u8>) {
        if !scheme.verify(self, &sig) {
            error!(target: "network", "Can't verify scheme");
            return
        }

        self.signature = Some(sig.clone());
        let raw = encode(self);
        self.raw = Some(raw);
    }

    pub fn reset_signature(&mut self) {
        self.signature = None;
        self.raw = None;
    }

    pub fn set_seq(&mut self, seq: u64) {
        self.seq = seq;
        self.signature = None;
        self.raw = None;
    }

    pub fn get_seq(&self) -> u64 {
        self.seq
    }
}

impl Encodable for Record {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(self.pairs.len() * 2 + 3)
            .append(&self.signature)
            .append(&self.seq)
            .append(&self.raw);

        for (key, value) in &self.pairs {
            s.append(key).append(value);
        }

        if s.len() > ENR_MAX_SIZE {
            error!(target: "network", "Encoded bytes exceed limit(300 bytes), length is {:?}", s.len())
        }
    }
}

impl Encodable for Value {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1);

        match self {
            Value::Tcp(tcp) => s.append(&tcp.0),
            Value::Tcp6(tcp6) => s.append(&tcp6.0),
            Value::Udp(udp) => s.append(&udp.0),
            Value::Udp6(udp6) => s.append(&udp6.0),
            Value::Id(id) => s.append(&id.0),
            Value::IpAddr(ip) => match ip {
                IpAddr::V4(ip4) => s.append_raw(&ip4.octets()[..], 4),
                IpAddr::V6(ip6) => s.append_raw(&ip6.octets()[..], 16),
            }
        };
    }
}

/// Tcp port of a node
#[derive(Clone, PartialEq, Debug)]
pub struct Tcp(pub u16);

impl Entry for Tcp {
    fn enr_key(&self) -> String {
        "tcp".to_owned()
    }
}

/// Tcp version 6 port of a node
#[derive(Clone, PartialEq, Debug)]
pub struct Tcp6(pub u16);

impl Entry for Tcp6 {
    fn enr_key(&self) -> String {
        "tcp6".to_owned()
    }
}

/// Udp port of a node
#[derive(Clone, PartialEq, Debug)]
pub struct Udp(pub u16);

impl Entry for Udp {
    fn enr_key(&self) -> String {
        "udp".to_owned()
    }
}

/// Udp version 6 port of a node
#[derive(Clone, PartialEq, Debug)]
pub struct Udp6(pub u16);

impl Entry for Udp6 {
    fn enr_key(&self) -> String {
        "udp6".to_owned()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Id(pub String);

pub const ID_V4: &str  = "v4";

impl Entry for Id {
    fn enr_key(&self) -> String {
        "id".to_owned()
    }
}

impl Entry for IpAddr {
    fn enr_key(&self) -> String {
        if self.is_ipv4() {
            "ip".to_owned()
        } else {
            "ip6".to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEnr {}

    impl Scheme for TestEnr {
        fn verify(&self, r: &Record, sig: &[u8]) -> bool {
            let id = Id("".to_owned());
            let id = match r.get(&id).unwrap() {
                Value::Id(id) => id.clone(),
                _ => panic!("Can't get Id")
            };

            sig == make_test_signature(id, r.seq).as_slice()
        }
    }

    impl Entry for TestEnr {
        fn enr_key(&self) -> String {
            "test_scheme".to_owned()
        }
    }

    struct TestEnr2 {}

    impl Scheme for TestEnr2 {
        fn verify(&self, r: &Record, sig: &[u8]) -> bool {
            let id = Id("".to_owned());
            let id = match r.get(&id).unwrap() {
                Value::Id(id) => id.clone(),
                _ => panic!("Can't get Id")
            };

            sig == make_test_signature(id, r.seq).as_slice()
        }
    }

    impl Entry for TestEnr2 {
        fn enr_key(&self) -> String {
            "test_scheme_2".to_owned()
        }
    }

    fn make_test_signature(id: Id, seq: u64) -> Vec<u8> {
        let mut sig = Vec::new();
        sig.extend_from_slice(&seq.to_be_bytes());
        sig.extend_from_slice(id.0.as_ref());
        sig
    }

    fn new_scheme_map<T: Scheme>(id: String, scheme: T) -> SchemeMap<T> {
        let mut map = HashMap::new();
        map.insert(id, scheme);
        SchemeMap(map)
    }

    #[test]
    fn test_get_set_id() {
        let id = Value::Id(Id("test_id".to_owned()));
        let mut r = Record::default();

        r.set(id.clone());

        let id2 = Id("".to_owned());
        let id2 = r.get(&id2).unwrap();
        assert_eq!(id, *id2)
    }

    #[test]
    fn test_get_set_ipv4() {
        let ip = Value::IpAddr("127.0.0.1".parse().unwrap());
        let mut r = Record::default();

        r.set(ip.clone());

        let ip2 = Value::IpAddr("0.0.0.0".parse().unwrap());
        let ip2 = r.get(&ip2).unwrap();
        assert_eq!(ip, *ip2)
    }

    #[test]
    fn test_get_set_udp() {
        let udp = Value::Udp(Udp(12345));
        let mut r = Record::default();

        r.set(udp.clone());

        let udp2 = Udp(11);
        let udp2 = r.get(&udp2).unwrap();
        assert_eq!(udp, *udp2)
    }

    enum Enr {
        TestEnr(TestEnr),
        TestEnr2(TestEnr2)
    }

    impl Scheme for Enr {
        fn verify(&self, record: &Record, sig: &[u8]) -> bool {
            match self {
                Enr::TestEnr(enr) => enr.verify(record, sig),
                Enr::TestEnr2(enr) => enr.verify(record, sig),
            }
        }
    }

    #[test]
    fn test_scheme() {
        let test_enr = TestEnr {};
        let test_enr2 = TestEnr2 {};
        let mut map = new_scheme_map("test_scheme".to_owned(),Enr::TestEnr(test_enr));
        map.add("test_scheme_2".to_owned(), Enr::TestEnr2(test_enr2));

        let mut record = Record::default();
        let mut record2 = Record::default();
        record.set(Value::Id(Id("test_scheme".to_owned())));
        record2.set(Value::Id(Id("test_scheme_2".to_owned())));

        let test_scheme = record.scheme().unwrap();
        let test_scheme_2 = record2.scheme().unwrap();

        assert_eq!("test_scheme", &test_scheme);
        assert_eq!("test_scheme_2", &test_scheme_2)
    }

    #[test]
    fn test_signature() {
        let test_enr = TestEnr {};

        let mut record = Record::default();
        record.set(Value::Id(Id("test_scheme".to_owned())));

        let test_sig = make_test_signature(Id("test_scheme".to_owned()), record.seq);
        record.set_signature(&test_enr, &test_sig);

        assert_eq!(test_sig, record.signature.unwrap());
    }

    #[test]
    fn test_verify() {
        let test_enr = TestEnr {};
        let map = new_scheme_map("test_scheme".to_owned(),Enr::TestEnr(test_enr.clone()));

        let mut record = Record::default();
        record.set(Value::Id(Id("test_scheme".to_owned())));

        let test_sig = make_test_signature(Id("test_scheme".to_owned()), record.seq);
        record.set_signature(&test_enr, &test_sig);

        let scheme = map.0.get("test_scheme").unwrap();

        assert!(scheme.verify(&record, &record.signature.clone().unwrap()))
    }
}
