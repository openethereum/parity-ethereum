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
  blocks: {},
  transactions: {},
  bytecodes: {},
  methods: {},
  accounts: {},
  contracts: {}
};

const initialContractState = {
  blockSubscriptionId: -1,
  subscriptionId: -1,
  events: {
    mined: [],
    pending: [],
    loading: true
  },
  queries: {}
};

export default handleActions({
  setBlock (state, action) {
    const { blockNumber, block } = action;

    const blocks = Object.assign({}, state.blocks, {
      [blockNumber.toString()]: block
    });

    return Object.assign({}, state, { blocks });
  },

  setBlocksPending (state, action) {
    const { blockNumbers, pending } = action;
    const blocks = Object.assign({}, state.blocks);

    blockNumbers.forEach(blockNumber => {
      const key = blockNumber.toString();
      const block = blocks[key];
      blocks[key] = {
        ...block,
        pending
      };
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

  setTransactionsPending (state, action) {
    const { txHashes, pending } = action;
    const transactions = Object.assign({}, state.transactions);

    txHashes.forEach(hash => {
      const transaction = transactions[hash];
      transactions[hash] = {
        ...transaction,
        pending
      };
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
  },

  setAccount (state, action) {
    const { address, info } = action;
    const { accounts } = state;

    const account = accounts[address];

    const newAccounts = {
      ...accounts,
      [address]: {
        ...account,
        ...info
      }
    };

    return Object.assign({}, state, { accounts: newAccounts });
  },

  setContract (state, action) {
    const { address, info } = action;
    const { contracts } = state;

    const contract = contracts[address];

    const newContracts = {
      ...contracts,
      [address]: {
        ...initialContractState,
        ...contract,
        ...info
      }
    };

    return Object.assign({}, state, { contracts: newContracts });
  },

  updateContractEvents (state, action) {
    const { address, events } = action;
    const { contracts } = state;

    const contract = contracts[address];
    const prevEvents = contract.events;

    const prevPendingEvents = prevEvents.pending
      .filter((pending) => {
        return !events
          .find((event) => {
            const isMined = (event.state === 'mined') && (event.transactionHash === pending.transactionHash);
            const isPending = (event.state === 'pending') && (event.key === pending.key);

            return isMined || isPending;
          });
      });

    const prevMinedEvents = prevEvents.mined
      .filter((mined) => {
        const txHash = mined.transactionHash;
        return !events.find((event) => event.transactionHash === txHash);
      });

    const minedEvents = events
      .filter((event) => event.state === 'mined')
      .concat(prevMinedEvents)
      .sort(sortEvents);

    const pendingEvents = events
      .filter((event) => event.state === 'pending')
      .concat(prevPendingEvents)
      .sort(sortEvents);

    const newContracts = {
      ...contracts,
      [address]: {
        ...contract,
        events: {
          pending: pendingEvents,
          mined: minedEvents,
          loading: false
        }
      }
    };

    return Object.assign({}, state, { contracts: newContracts });
  }

}, initialState);

function sortEvents (a, b) {
  return b.blockNumber.cmp(a.blockNumber) || b.logIndex.cmp(a.logIndex);
}
