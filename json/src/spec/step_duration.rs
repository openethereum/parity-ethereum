// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Step duration configuration parameter

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::uint::Uint;

/// Step duration can be specified either as a `Uint` (in seconds), in which case it will be
/// constant, or as a list of pairs consisting of a timestamp of type `Uint` and a duration, in
/// which case the duration of a step will be determined by a mapping arising from that list.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum StepDuration {
	/// Duration of all steps.
	Single(Uint),
	/// Step duration transitions: a mapping of timestamp to step durations.
	Transitions(BTreeMap<Uint, Uint>),
}
