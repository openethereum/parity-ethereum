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

//! Spec deserialization.

pub mod account;
pub mod authority_round;
pub mod basic_authority;
pub mod builtin;
pub mod clique;
pub mod engine;
pub mod ethash;
pub mod genesis;
pub mod instant_seal;
pub mod null_engine;
pub mod params;
pub mod seal;
pub mod spec;
pub mod state;
pub mod validator_set;

pub use self::{
    account::Account,
    authority_round::{AuthorityRound, AuthorityRoundParams},
    basic_authority::{BasicAuthority, BasicAuthorityParams},
    builtin::{Builtin, Linear, Pricing},
    clique::{Clique, CliqueParams},
    engine::Engine,
    ethash::{BlockReward, Ethash, EthashParams},
    genesis::Genesis,
    instant_seal::{InstantSeal, InstantSealParams},
    null_engine::{NullEngine, NullEngineParams},
    params::Params,
    seal::{AuthorityRoundSeal, Ethereum, Seal, TendermintSeal},
    spec::{ForkSpec, Spec},
    state::State,
    validator_set::ValidatorSet,
};
