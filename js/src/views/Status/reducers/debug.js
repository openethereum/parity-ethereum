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
import { union } from 'lodash';

const initialState = {
  levels: '',
  logging: true,
  logs: []
};

const maxLogs = 1024;

export const actionHandlers = {

  'update devLogsLevels' (state, action) {
    return {
      ...state,
      levels: `${action.payload}`
    };
  },

  'remove devLogs' (state, action) {
    return {
      ...state,
      logs: []
    };
  },

  'update devLogging' (state, action) {
    return {
      ...state,
      logging: action.payload
    };
  },

  'update devLogs' (state, action) {
    if (!state.logging) {
      return { ...state };
    }

    let newLogs = union(state.logs, action.payload.reverse());

    return {
      ...state,
      logs: newLogs.slice(newLogs.length - maxLogs)
    };
  }

};

export default handleActions(actionHandlers, initialState);
