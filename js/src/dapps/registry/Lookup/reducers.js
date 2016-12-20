// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
  name: '', type: '',
  result: null
};

export default (state = initialState, action) => {
  if (action.type.slice(0, 7) !== 'lookup ') {
    return state;
  }

  if (isStage('clear', action)) {
    return { ...state, result: null };
  }

  if (isStage('start', action)) {
    return {
      pending: true,
      name: action.name, type: action.entry,
      result: null
    };
  }

  if (isStage('error', action)) {
    return {
      pending: false,
      name: initialState.name, type: initialState.type,
      result: null
    };
  }

  if (isStage('success', action)) {
    return {
      pending: false,
      name: initialState.name, type: initialState.type,
      result: action.result
    };
  }

  return state;
};
