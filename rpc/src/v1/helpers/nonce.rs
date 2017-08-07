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

use std::sync::Arc;
use futures::{Future, Poll, Async};
use util::{U256, RwLock};
use std::collections::{VecDeque};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NonceError {
    Dropped,
    InvalidPoll,
}

#[derive(Copy, Clone)]
enum NonceState {
    Prospective,
    Reserved,
    Dispatch,
    Error(NonceError),
}

pub struct Nonce {
    state: NonceState,
    value: U256,
    reserved: Arc<RwLock<VecDeque<U256>>>,
}

impl Drop for Nonce {
    fn drop(&mut self) {
        match self.state {
            NonceState::Reserved => {
                let reserved = self.reserved.read();
                if let Some(idx) = reserved.iter().position(|x| *x == self.value) {
                    drop(reserved);
                    let mut reserved = self.reserved.write();
                    reserved.drain(idx..); // any reservations after this one are now invalid
                }
            }
            _ => {}
        }
    }
}

impl Future for Nonce {
    type Item = U256;
    type Error = NonceError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state {
            NonceState::Prospective => {
                let result = if let Some(reserved) = self.reserved.try_read() {
                    while reserved.contains(&self.value) { // reserved by another thread
                        self.value = self.value + U256::one(); // increment prospective nonce
                    }
                    drop(reserved);
                    // prospective nonce is available
                    if let Some(mut reserved) = self.reserved.try_write() {
                        reserved.push_back(self.value);
                        // use this nonce for signing
                        // then poll for dispatch readiness
                        // dispatch may fail if another thread drops its nonce
                        Ok(Async::Ready(self.value))
                    } else {
                        Ok(Async::NotReady)
                    }
                } else {
                    Ok(Async::NotReady)
                };

                self.state = match result {
                    Ok(Async::Ready(_)) => NonceState::Reserved,
                    Ok(Async::NotReady) => NonceState::Prospective,
                    Err(e) => NonceState::Error(e),
                };

                result
            }

            NonceState::Reserved => {
                let result = if let Some(mut reserved) = self.reserved.try_write() { 
                    if reserved.front().map(|&x| x == self.value).is_some() { // front of the line, ready for dispatch
                        let _ = reserved.pop_front();
                        Ok(Async::Ready(self.value))
                    } else if reserved.contains(&self.value) { // still in line, not ready
                        Ok(Async::NotReady)
                    } else { // dropped, will never be ready
                        Err(NonceError::Dropped)
                    }
                } else { // queue locked, not ready
                    Ok(Async::NotReady)
                };

                self.state = match result {
                    Ok(Async::Ready(_)) => NonceState::Dispatch,
                    Ok(Async::NotReady) => NonceState::Reserved,
                    Err(e) => NonceState::Error(e),
                };

                result
            }

            NonceState::Dispatch => Ok(Async::Ready(self.value)),
            NonceState::Error(ne) => Err(ne)
        }
    }
}

impl Nonce {
    pub fn start_with(nonce: U256, reserved: Arc<RwLock<VecDeque<U256>>>) -> Nonce {
        Nonce {
            state: NonceState::Prospective,
            value: nonce,
            reserved: reserved,
        }
    }

    pub fn poll_reserve(&mut self) -> Result<Async<U256>, NonceError> {
        match self.state {
            NonceState::Prospective => self.poll(),
            NonceState::Reserved => Ok(Async::Ready(self.value)),
            _ => Err(NonceError::InvalidPoll),
        }
    }

    pub fn poll_dispatch(&mut self) -> Result<Async<U256>, NonceError> {
        match self.state {
            NonceState::Reserved => self.poll(),
            NonceState::Dispatch => Ok(Async::Ready(self.value)),
            _ => Err(NonceError::InvalidPoll),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve() {
        let mut reserved_inner = VecDeque::new();
        reserved_inner.push_back(U256::from(0));
        reserved_inner.push_back(U256::from(1));
        let reserved = Arc::new(RwLock::new(reserved_inner));

        let mut nonce_a = Nonce::start_with(U256::from(0), reserved.clone());
        let mut nonce_b = Nonce::start_with(U256::from(3), reserved.clone());
        let mut nonce_e = Nonce::start_with(U256::from(3), reserved.clone());
        let mut nonce_l = Nonce::start_with(U256::from(3), reserved.clone());

        let reserve_a = nonce_a.poll_reserve().ok().unwrap();
        let reserve_b = nonce_b.poll_reserve().ok().unwrap();
        let reserve_e = nonce_e.poll_dispatch().err().unwrap();

        let read = reserved.read();
        let reserve_l = nonce_l.poll_reserve().ok().unwrap();

        assert_eq!(reserve_a, Async::Ready(U256::from(2)));
        assert_eq!(reserve_b, Async::Ready(U256::from(3)));
        assert_eq!(reserve_e, NonceError::InvalidPoll);
        assert_eq!(reserve_l, Async::NotReady);
    }

    #[test]
    fn dispatch() {
        let mut reserved = Arc::new(RwLock::new(VecDeque::new()));
        let mut nonce = Nonce::start_with(U256::from(55), reserved.clone());
        let mut nonce_l = Nonce::start_with(U256::from(55), reserved.clone());
        
        let _ = nonce.poll_reserve();
        let _ = nonce_l.poll_reserve();

        let dispatch = nonce.poll_dispatch().ok().unwrap();
        let dispatch_e = nonce.poll_reserve().err().unwrap();

        let read = reserved.read();
        let dispatch_l = nonce_l.poll_dispatch().ok().unwrap();

        assert_eq!(dispatch, Async::Ready(U256::from(55)));
        assert_eq!(dispatch_e, NonceError::InvalidPoll);
        assert_eq!(dispatch_l, Async::NotReady);
    }
}