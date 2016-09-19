// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
  devLogs: [],
  devLogsLevels: null,
  devLogsEnabled: false
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
  }
}, initialState);
