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

//! Consensus engine specification and basic implementations.

mod authority_round;
mod basic_authority;
mod clique;
mod ethash;
mod instant_seal;
mod null_engine;
mod validator_set;

pub mod block_reward;

pub use self::authority_round::AuthorityRound;
pub use self::basic_authority::BasicAuthority;
pub use self::instant_seal::{InstantSeal, InstantSealParams};
pub use self::null_engine::NullEngine;
pub use self::clique::Clique;
pub use self::ethash::{Ethash, Seal as EthashSeal};
