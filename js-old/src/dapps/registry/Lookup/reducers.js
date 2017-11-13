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

import { isStage } from '../util/actions';

const initialState = {
  pending: false,
  result: null
};

export default (state = initialState, action) => {
  const { type } = action;

  if (!/^(lookup|reverseLookup|ownerLookup)/.test(type)) {
    return state;
  }

  if (isStage('clear', action)) {
    return { pending: state.pending, result: null };
  }

  if (isStage('start', action)) {
    return { pending: true, result: null };
  }

  if (isStage('error', action)) {
    return { pending: false, result: null };
  }

  if (isStage('success', action)) {
    return { pending: false, result: action.result };
  }

  return state;
};
