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

const initialState = {
  error: false,
  noOfErrors: 0,
  name: 'My node',
  bestBlock: 'loading...',
  hashrate: 'loading...',
  connectedPeers: 0,
  activePeers: 0,
  peers: 0,
  accounts: [],
  version: '-'
};

export default handleActions({
  error (state, action) {
    return {
      ...state,
      disconnected: (action.payload.message === 'Invalid JSON RPC response: ""'),
      noOfErrors: state.noOfErrors + 1
    };
  },

  'update blockNumber' (state, action) {
    return {
      ...resetError(state),
      bestBlock: `${action.payload}`
    };
  },

  'update hashrate' (state, action) {
    return {
      ...resetError(state),
      hashrate: `${action.payload}`
    };
  },

  'update netPeers' (state, action) {
    return {
      ...state,
      connectedPeers: action.payload.connected,
      activePeers: action.payload.active
    };
  },

  'update version' (state, action) {
    return {
      ...resetError(state),
      version: action.payload
    };
  },

  'update accounts' (state, action) {
    return {
      ...resetError(state),
      accounts: action.payload
    };
  },

  'update nodeName' (state, action) {
    return {
      ...resetError(state),
      name: action.payload || ' '
    };
  }

}, initialState);

function resetError (state) {
  return {
    ...state,
    disconnected: false,
    noOfErrors: 0
  };
}
