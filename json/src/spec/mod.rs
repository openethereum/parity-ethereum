// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Spec deserialization.

pub mod account;
pub mod builtin;
pub mod genesis;
pub mod params;
pub mod spec;
pub mod seal;
pub mod engine;
pub mod state;
pub mod ethash;
pub mod validator_set;
pub mod basic_authority;
pub mod authority_round;
pub mod tendermint;
pub mod null_engine;
pub mod instant_seal;
pub mod hardcoded_sync;

pub use self::account::Account;
pub use self::builtin::{Builtin, Pricing, Linear};
pub use self::genesis::Genesis;
pub use self::params::Params;
pub use self::spec::{Spec, ForkSpec};
pub use self::seal::{Seal, Ethereum, AuthorityRoundSeal, TendermintSeal};
pub use self::engine::Engine;
pub use self::state::State;
pub use self::ethash::{Ethash, EthashParams, BlockReward};
pub use self::validator_set::ValidatorSet;
pub use self::basic_authority::{BasicAuthority, BasicAuthorityParams};
pub use self::authority_round::{AuthorityRound, AuthorityRoundParams};
pub use self::tendermint::{Tendermint, TendermintParams};
pub use self::null_engine::{NullEngine, NullEngineParams};
pub use self::instant_seal::{InstantSeal, InstantSealParams};
pub use self::hardcoded_sync::HardcodedSync;
