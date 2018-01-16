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

import BigNumber from 'bignumber.js';
import { handleActions } from 'redux-actions';

const DEFAULT_NETCHAIN = '(unknown)';
const DEFAULT_STATUS = 'needsAttention';
const initialState = {
  blockNumber: new BigNumber(0),
  blockTimestamp: new Date(),
  clientVersion: '',
  gasLimit: new BigNumber(0),
  health: {
    peers: {
      status: DEFAULT_STATUS
    },
    sync: {
      status: DEFAULT_STATUS
    },
    time: {
      status: DEFAULT_STATUS
    },
    overall: {
      isNotReady: true,
      status: DEFAULT_STATUS,
      message: []
    }
  },
  netChain: DEFAULT_NETCHAIN,
  netPeers: {
    active: new BigNumber(0),
    connected: new BigNumber(0),
    max: new BigNumber(0),
    peers: []
  },
  netVersion: '0',
  nodeKind: null,
  nodeKindFull: null,
  syncing: null,
  isConnected: false,
  isConnecting: false,
  isTest: undefined,
  traceMode: undefined
};

export default handleActions({
  statusBlockNumber (state, action) {
    const { blockNumber } = action;

    return Object.assign({}, state, { blockNumber });
  },

  statusCollection (state, action) {
    const { collection } = action;

    return Object.assign({}, state, collection);
  }
}, initialState);

export {
  DEFAULT_NETCHAIN,
  initialState
};
