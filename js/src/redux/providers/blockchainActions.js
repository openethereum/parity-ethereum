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

import { uniq } from 'lodash';

import Contracts from '../../contracts';
import etherscan from '../../3rdparty/etherscan';

export function setBlock (blockNumber, block) {
  return {
    type: 'setBlock',
    blockNumber, block
  };
}

export function setTransaction (txHash, info) {
  return {
    type: 'setTransaction',
    txHash, info
  };
}

export function setBytecode (address, bytecode) {
  return {
    type: 'setBytecode',
    address, bytecode
  };
}

export function setMethod (signature, method) {
  return {
    type: 'setMethod',
    signature, method
  };
}

export function setAccount (address, info) {
  return {
    type: 'setAccount',
    address, info
  };
}

export function fetchAccountTransactions (address) {
  return (dispatch, getState) => {
    dispatch(setAccount(address, { loading: true, error: null }));

    const state = getState();

    const { api } = state;
    const { traceMode, isTest } = state.nodeStatus;

    const transactionsPromise = false
      ? fetchTraceTransactions(api, address)
      : fetchEtherscanTransactions(isTest, address);

    transactionsPromise
      .then((transactions) => {
        dispatch(setAccount(address, {
          loading: false,
          transactions
        }));

        const blockNumbers = uniq(transactions.map(tx => tx.blockNumber));
        const txHashes = uniq(transactions.map(tx => tx.hash));

        blockNumbers.forEach(blockNumber => dispatch(fetchBlock(blockNumber)));
        txHashes.forEach(hash => dispatch(fetchTransaction(hash)));
      })
      .catch((e) => {
        console.error('::fetchAccountTransactions', address, e);
        dispatch(setAccount(address, {
          loading: false,
          error: e
        }));
      });
  };
}

export function fetchBlock (blockNumber) {
  return (dispatch, getState) => {
    const { blocks } = getState().blockchain;

    if (blocks[blockNumber.toString()]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getBlockByNumber(blockNumber)
      .then(block => {
        dispatch(setBlock(blockNumber, block));
      })
      .catch(e => {
        console.error('blockchain::fetchBlock', e);
      });
  };
}

export function fetchTransaction (txHash) {
  return (dispatch, getState) => {
    const { transactions } = getState().blockchain;

    if (transactions[txHash]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getTransactionByHash(txHash)
      .then(info => {
        dispatch(setTransaction(txHash, info));
      })
      .catch(e => {
        console.error('blockchain::fetchTransaction', e);
      });
  };
}

export function fetchBytecode (address) {
  return (dispatch, getState) => {
    const { bytecodes } = getState().blockchain;

    if (bytecodes[address]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getCode(address)
      .then(code => {
        dispatch(setBytecode(address, code));
      })
      .catch(e => {
        console.error('blockchain::fetchBytecode', e);
      });
  };
}

export function fetchMethod (signature) {
  return (dispatch, getState) => {
    const { methods } = getState().blockchain;

    if (methods[signature]) {
      return;
    }

    Contracts
      .get()
      .signatureReg.lookup(signature)
      .then(method => {
        dispatch(setMethod(signature, method));
      })
      .catch(e => {
        console.error('blockchain::fetchMethod', e);
      });
  };
}

export function fetchEtherscanTransactions (isTest, address) {
  return etherscan.account
    .transactions(address, 0, isTest);
}

export function fetchTraceTransactions (api, address) {
  return Promise
    .all([
      api.trace.filter({
        fromBlock: 0,
        fromAddress: address
      }),
      api.trace.filter({
        fromBlock: 0,
        toAddress: address
      })
    ])
    .then(([fromTransactions, toTransactions]) => {
      const transactions = [].concat(fromTransactions, toTransactions);

      return transactions.map(transaction => ({
        from: transaction.action.from,
        to: transaction.action.to,
        blockNumber: transaction.blockNumber,
        hash: transaction.transactionHash
      }));
    });
}
