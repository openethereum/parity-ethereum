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

const isProd = process.env.NODE_ENV === 'production';

const initialState = {
  isLoading: true,
  isNodeRunning: true,
  isConnected: false,
  logging: false && !isProd,
  token: '',
  url: window.location.host,
  proxyUrl: 'http://localhost:8080/proxy/proxy.pac'
};

export default handleActions({

  'update isConnected' (state, action) {
    const isDisconnected = state.isConnected && !action.payload;
    return {
      ...state,
      isLoading: false,
      isConnected: action.payload,
      // if we are disconnected assume automatically that node is down
      isNodeRunning: !isDisconnected && state.isNodeRunning
    };
  },

  'update isNodeRunning' (state, action) {
    const isRunning = action.payload;
    const goesOnline = isRunning && !state.isNodeRunning;

    return {
      ...state,
      isNodeRunning: isRunning,
      // if node is down assume automatically that we are not connected
      isLoading: goesOnline || (isRunning && state.isLoading),
      isConnected: isRunning && state.isConnected
    };
  },

  'update logging' (state, action) {
    return {
      ...state,
      logging: action.payload
    };
  },

  'update url' (state, action) {
    return {
      ...state,
      url: action.payload
    };
  },

  'update proxy' (state, action) {
    return {
      ...state,
      proxyUrl: action.payload
    };
  },

  'update token' (state, action) {
    return {
      ...state,
      token: action.payload
    };
  }

}, initialState);
