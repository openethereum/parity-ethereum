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
  blocks: {},
  transactions: {},
  bytecodes: {},
  methods: {}
};

export default handleActions({
  setBlock (state, action) {
    const { blockNumber, block } = action;

    const blocks = Object.assign({}, state.blocks, {
      [blockNumber.toString()]: block
    });

    return Object.assign({}, state, { blocks });
  },

  setTransaction (state, action) {
    const { txHash, info } = action;

    const transactions = Object.assign({}, state.transactions, {
      [txHash]: info
    });

    return Object.assign({}, state, { transactions });
  },

  setBytecode (state, action) {
    const { address, bytecode } = action;

    const bytecodes = Object.assign({}, state.bytecodes, {
      [address]: bytecode
    });

    return Object.assign({}, state, { bytecodes });
  },

  setMethod (state, action) {
    const { signature, method } = action;

    const methods = Object.assign({}, state.methods, {
      [signature]: method
    });

    return Object.assign({}, state, { methods });
  }
}, initialState);
