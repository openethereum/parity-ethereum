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

import { isAction, isStage } from '../util/actions';

const initialState = {
  error: null,
  pending: false,
  name: '', type: '', value: ''
};

export default (state = initialState, action) => {
  switch (action.type) {
    case 'clearError':
      return {
        ...state,
        error: null
      };
  }

  if (!isAction('records', 'update', action)) {
    return state;
  }

  if (isStage('start', action)) {
    return {
      ...state, pending: true,
      error: null,
      name: action.name, type: action.key, value: action.value
    };
  }

  if (isStage('success', action) || isStage('fail', action)) {
    return {
      ...state, pending: false,
      error: action.error || null,
      name: initialState.name, type: initialState.type, value: initialState.value
    };
  }

  return state;
};
