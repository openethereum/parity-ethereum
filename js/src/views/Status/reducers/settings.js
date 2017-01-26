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
  chain: 'loading...',
  networkPort: 0,
  maxPeers: 0,
  rpcEnabled: false,
  rpcInterface: '-',
  rpcPort: 0
};

export default handleActions({
  'update netChain' (state, action) {
    return {
      ...state,
      chain: action.payload
    };
  },

  'update netPort' (state, action) {
    return {
      ...state,
      networkPort: action.payload
    };
  },

  'update netPeers' (state, action) {
    return {
      ...state,
      maxPeers: action.payload.max
    };
  },

  'update rpcSettings' (state, action) {
    const rpc = action.payload;

    return {
      ...state,
      rpcEnabled: rpc.enabled,
      rpcInterface: rpc.interface,
      rpcPort: rpc.port
    };
  }
}, initialState);
