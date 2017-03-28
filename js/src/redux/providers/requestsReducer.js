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

import { handleActions } from 'redux-actions';

const initialState = {};

export default handleActions({
  setRequest (state, action) {
    const { requestId, requestData } = action;

    const nextState = {
      ...state,
      [requestId]: {
        ...(state[requestId] || {}),
        ...requestData
      }
    };

    return nextState;
  },

  deleteRequest (state, action) {
    const { requestId } = action;
    const nextState = { ...state };

    delete nextState[requestId];
    return nextState;
  }
}, initialState);
