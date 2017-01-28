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
import rpcMetods from '../data/rpc.json';

const initialState = {
  prevCalls: [],
  callNo: 1,
  selectedMethod: rpcMetods.methods[0]
};

export const actionHandlers = {

  'add rpcResponse' (state, action) {
    const calls = [action.payload].concat(state.prevCalls);
    const maxCalls = 64;

    return {
      ...state,
      callNo: state.callNo + 1,
      prevCalls: calls.slice(0, maxCalls)
    };
  },

  'sync rpcStateFromLocalStorage' (state, action) {
    return {
      ...state,
      prevCalls: action.payload.prevCalls,
      callNo: action.payload.callNo,
      selectedMethod: action.payload.selectedMethod
    };
  },

  'reset rpcPrevCalls' (state, action) {
    return {
      ...state,
      callNo: 1,
      prevCalls: []
    };
  },

  'select rpcMethod' (state, action) {
    return {
      ...state,
      selectedMethod: action.payload
    };
  }

};

export default handleActions(actionHandlers, initialState);
