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

export default {
  acceptNonReserved: {
    label: `Accept non-reserved`
  },
  acceptNonReservedPeers: {
    success: `Accepting non-reserved peers`
  },
  addReserved: {
    label: `Add reserved`
  },
  dropNonReserved: {
    label: `Drop non-reserved`
  },
  dropNonReservedPeers: {
    success: `Dropping non-reserved peers`
  },
  form: {
    action: {
      label: `{add, select, true {Add} false {}}{remove, select, true {Remove} false {}}`,
      success: `Successfully {add, select, true {added} false {}}{remove, select, true {removed} false {}} a reserved peer`
    },
    cancel: {
      label: `Cancel`
    },
    label: `Peer enode URL`
  },
  removeReserved: {
    label: `Remove reserved`
  }
};
