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
const initialState = {
  blockNumber: new BigNumber(0),
  blockTimestamp: new Date(),
  devLogs: [],
  devLogsLevels: null,
  devLogsEnabled: false,
  clientVersion: '',
  coinbase: '',
  defaultExtraData: '',
  enode: '',
  extraData: '',
  gasFloorTarget: new BigNumber(0),
  gasLimit: new BigNumber(0),
  hashrate: new BigNumber(0),
  minGasPrice: new BigNumber(0),
  netChain: DEFAULT_NETCHAIN,
  netPeers: {
    active: new BigNumber(0),
    connected: new BigNumber(0),
    max: new BigNumber(0)
  },
  netPort: new BigNumber(0),
  netVersion: '0',
  rpcSettings: {},
  syncing: true,
  isConnected: false,
  isConnecting: false,
  isTest: undefined,
  refreshStatus: false,
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
  },

  statusLogs (state, action) {
    const { logInfo } = action;

    return Object.assign({}, state, logInfo);
  },

  toggleStatusLogs (state, action) {
    const { devLogsEnabled } = action;

    return Object.assign({}, state, { devLogsEnabled });
  },

  clearStatusLogs (state, action) {
    return Object.assign({}, state, { devLogs: [] });
  },

  toggleStatusRefresh (state, action) {
    const { refreshStatus } = action;

    return Object.assign({}, state, { refreshStatus });
  }
}, initialState);

export {
  DEFAULT_NETCHAIN,
  initialState
};
