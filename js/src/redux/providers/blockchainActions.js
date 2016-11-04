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

import { uniq, uniqWith } from 'lodash';

import Contracts from '../../contracts';
import etherscan from '../../3rdparty/etherscan';

export function setBlock (blockNumber, block) {
  return {
    type: 'setBlock',
    blockNumber, block
  };
}

export function setBlocks (blocks, extra) {
  return {
    type: 'setBlocks',
    blocks, extra
  };
}

export function setBlocksPending (blockNumbers, pending) {
  return {
    type: 'setBlocksPending',
    blockNumbers, pending
  };
}

export function setTransaction (txHash, info) {
  return {
    type: 'setTransaction',
    txHash, info
  };
}

export function setTransactions (transactions, extra) {
  return {
    type: 'setTransactions',
    transactions, extra
  };
}

export function clearTransactions (txHashes) {
  return {
    type: 'clearTransactions',
    txHashes
  };
}

export function setTransactionsPending (txHashes, pending) {
  return {
    type: 'setTransactionsPending',
    txHashes, pending
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

export function setContract (address, info) {
  return {
    type: 'setContract',
    address, info
  };
}

export function fetchAccountTransactions (address) {
  return (dispatch, getState) => {
    dispatch(setAccount(address, { loading: true, error: null }));

    const state = getState();

    const { api } = state;
    const { traceMode, isTest } = state.nodeStatus;

    const transactionsPromise = traceMode
      ? fetchTraceTransactions(api, address)
      : fetchEtherscanTransactions(isTest, address);

    transactionsPromise
      .then((transactions) => {
        dispatch(setAccount(address, {
          loading: false,
          transactions
        }));

        // Load the corresponding blocks and transactions
        const blockNumbers = transactions.map(tx => tx.blockNumber);
        const txHashes = transactions.map(tx => tx.hash);

        dispatch(fetchBlocks(blockNumbers));
        dispatch(fetchTransactions(txHashes));
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

export function attachContract (address) {
  return (dispatch, getState) => {
    const state = getState();

    const { api, blockchain } = state;

    if (blockchain.contracts[address]) {
      return;
    }

    const { contracts } = state.personal;

    const contract = contracts[address];

    if (!contract) {
      return dispatch(setContract(address, {
        error: 'No contract found'
      }));
    }

    const instance = api.newContract(contract.meta.abi, address);
    dispatch(setContract(address, { ...contract, instance, eventsLoading: true }));
  };
}

export function subscribeToContractEvents (address) {
  return (dispatch, getState) => {
    const { api, blockchain } = getState();

    if (blockchain.contracts[address] && blockchain.contracts[address].subscriptionId > -1) {
      return null;
    }

    const { instance } = blockchain.contracts[address];

    instance
      .subscribe(
        null, {
          limit: 25,
          fromBlock: 0, toBlock: 'pending'
        }, (error, logs) => {
          if (error) {
            return console.error('::attachContract', address, error);
          }

          const events = logs.map((log) => logToEvent(api, log));
          dispatch(updateContractEvents(address, events));

          // Load the corresponding blocks and transactions
          const blockNumbers = events.map(e => e.blockNumber);
          const txHashes = events.map(e => e.transactionHash);

          const blocksP = getFetchBlocks(dispatch, getState, blockNumbers);
          const txsP = getFetchTransaction(dispatch, getState, txHashes);

          Promise
            .all([ blocksP, txsP ])
            .then(() => {
              dispatch(setContract(address, { eventsLoading: false }));
            });
        }
      )
      .then((subscriptionId) => {
        dispatch(setContract(address, { subscriptionId }));
      })
      .catch((e) => {
        throw e;
      });
  };
}

export function subscribeToContractQueries (address) {
  return (dispatch, getState) => {
    const { api, blockchain } = getState();

    if (blockchain.contracts[address] && blockchain.contracts[address].blockSubscriptionId > -1) {
      return null;
    }

    const { instance } = blockchain.contracts[address];

    const queries = instance.functions
      .filter((fn) => fn.constant)
      .filter((fn) => !fn.inputs.length);

    api
      .subscribe('eth_blockNumber', () => {
        Promise
          .all(queries.map((query) => query.call()))
          .then(results => {
            const values = queries.reduce((object, fn, idx) => {
              const key = fn.name;
              object[key] = results[idx];
              return object;
            }, {});

            dispatch(setContract(address, { queries: values }));
          })
          .catch((error) => {
            console.error('::subscribeToContractQueries::eth_blockNumber', address, error);
          });
      })
      .then((blockSubscriptionId) => {
        dispatch(setContract(address, { blockSubscriptionId }));
      });
  };
}

export function detachContract (address) {
  return (dispatch, getState) => {
    const state = getState();

    const { api } = state;
    const { contracts } = state.blockchain;

    const contract = contracts[address];

    if (!contract) {
      return;
    }

    const { subscriptionId, blockSubscriptionId } = contract;
    const promises = [];

    if (subscriptionId > -1) {
      promises.push(contract.instance.unsubscribe(subscriptionId));
    }

    if (blockSubscriptionId > -1) {
      promises.push(api.unsubscribe(blockSubscriptionId));
    }

    Promise.all(promises).then(() => {
      dispatch(clearContract(address));
    });
  };
}

export function clearContract (address) {
  return {
    type: 'clearContract',
    address
  };
}

export function updateContractEvents (address, events) {
  return {
    type: 'updateContractEvents',
    address, events
  };
}

export function fetchBlocks (blockNumbers) {
  return (dispatch, getState) => {
    getFetchBlocks(dispatch, getState, blockNumbers);
  };
}

function getFetchBlocks (dispatch, getState, blockNumbers) {
  const state = getState();
  const { blocks } = state.blockchain;

  const blocksToFetch = uniqWith(
    blockNumbers,
    (a, b) => a.equals(b)
  )
  .filter(blockNumber => {
    const key = blockNumber.toString();

    // If nothing in state
    if (!blocks[key]) return true;

    // If not pending or invalid
    return !blocks[key].pending && !blocks[key].valid;
  });

  if (blocksToFetch.length === 0) {
    return;
  }

  dispatch(setBlocksPending(blocksToFetch, true));

  return Promise
    .all(blocksToFetch.map(n => state.api.eth.getBlockByNumber(n)))
    .then(blocks => {
      dispatch(setBlocks(blocks, {
        pending: false,
        valid: true
      }));
    })
    .catch(e => {
      console.error('blockchain::fetchBlocks', e);

      blocksToFetch.forEach((blockNumber) => {
        dispatch(setBlock(blockNumber, { pending: false, valid: false }));
      });
    });
}

export function fetchBlock (blockNumber) {
  return (dispatch) => {
    dispatch(fetchBlocks([ blockNumber ]));
  };
}

export function fetchTransactions (txHashes) {
  return (dispatch, getState) => {
    getFetchTransaction(dispatch, getState, txHashes);
  };
}

function getFetchTransaction (dispatch, getState, txHashes) {
  const state = getState();
  const { transactions } = state.blockchain;

  const txsToFetch = uniq(txHashes)
    .filter(hash => {
      // If nothing in state
      if (!transactions[hash]) return true;

      // If not pending or invalid
      return !transactions[hash].pending && !transactions[hash].valid;
    });

  if (txsToFetch.length === 0) {
    return;
  }

  dispatch(setTransactionsPending(txsToFetch, true));

  return Promise
    .all(txsToFetch.map(h => state.api.eth.getTransactionByHash(h)))
    .then((transactions) => {
      dispatch(setTransactions(transactions, {
        pending: false,
        valid: true
      }));
    })
    .catch(e => {
      txsToFetch.forEach((txHash) => {
        dispatch(setTransaction(txHash, { pending: false, valid: false }));
      });

      throw e;
    });
}

export function fetchTransaction (txHash) {
  return (dispatch) => {
    dispatch(fetchTransactions([ txHash ]));
  };
}

export function fetchBytecode (address) {
  return (dispatch, getState) => {
    const { bytecodes } = getState().blockchain;

    const pending = bytecodes[address] && bytecodes[address].pending;
    const valid = bytecodes[address] && bytecodes[address].valid;

    if (pending || valid) {
      return;
    }

    dispatch(setBytecode(address, { pending: true }));

    const { api } = getState();

    api.eth
      .getCode(address)
      .then(code => {
        dispatch(setBytecode(address, {
          ...code,
          pending: false,
          valid: true
        }));
      })
      .catch(e => {
        console.error('blockchain::fetchBytecode', e);
        dispatch(setBytecode(address, { pending: true, valid: false }));
      });
  };
}

export function fetchMethod (signature) {
  return (dispatch, getState) => {
    const { methods } = getState().blockchain;

    const pending = methods[signature] && methods[signature].pending;
    const valid = methods[signature] && methods[signature].valid;

    if (pending || valid) {
      return;
    }

    dispatch(setMethod(signature, { pending: true }));

    Contracts
      .get()
      .signatureReg.lookup(signature)
      .then(method => {
        dispatch(setMethod(signature, {
          ...method,
          pending: false,
          valid: true
        }));
      })
      .catch(e => {
        console.error('blockchain::fetchMethod', e);
        dispatch(setMethod(signature, { pending: false, valid: false }));
      });
  };
}

function fetchEtherscanTransactions (isTest, address) {
  return etherscan.account
    .transactions(address, 0, isTest);
}

function fetchTraceTransactions (api, address) {
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

function logToEvent (api, log) {
  const key = api.util.sha3(JSON.stringify(log));
  const { address, blockNumber, logIndex, transactionHash, transactionIndex, params, type } = log;

  return {
    type: log.event,
    state: type,
    address,
    blockNumber,
    logIndex,
    transactionHash,
    transactionIndex,
    params,
    key
  };
}
