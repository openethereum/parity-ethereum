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

import { isEqual, uniq } from 'lodash';

import Contract from '~/api/contract';
import { bytesToHex, toHex } from '~/api/util/format';
import { ERROR_CODES } from '~/api/transport/error';
import { wallet as WALLET_ABI } from '~/contracts/abi';
import { MAX_GAS_ESTIMATION } from '~/util/constants';
import WalletsUtils from '~/util/wallets';

import { newError } from '~/ui/Errors/actions';

const UPDATE_OWNERS = 'owners';
const UPDATE_REQUIRE = 'require';
const UPDATE_DAILYLIMIT = 'dailylimit';
const UPDATE_TRANSACTIONS = 'transactions';
const UPDATE_CONFIRMATIONS = 'confirmations';

export function confirmOperation (address, owner, operation) {
  return modifyOperation('confirm', address, owner, operation);
}

export function revokeOperation (address, owner, operation) {
  return modifyOperation('revoke', address, owner, operation);
}

function modifyOperation (method, address, owner, operation) {
  return (dispatch, getState) => {
    const { api } = getState();
    const contract = new Contract(api, WALLET_ABI).at(address);

    const options = {
      from: owner,
      gas: MAX_GAS_ESTIMATION
    };

    const values = [ operation ];

    dispatch(setOperationPendingState(address, operation, true));

    contract.instance[method]
      .estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2);
        return contract.instance[method].postTransaction(options, values);
      })
      .then((requestId) => {
        return api
          .pollMethod('parity_checkRequest', requestId)
          .catch((e) => {
            dispatch(setOperationPendingState(address, operation, false));
            if (e.code === ERROR_CODES.REQUEST_REJECTED) {
              return;
            }

            throw e;
          });
      })
      .catch((error) => {
        dispatch(setOperationPendingState(address, operation, false));
        dispatch(newError(error));
      });
  };
}

export function attachWallets (_wallets) {
  return (dispatch, getState) => {
    const { wallet, api } = getState();

    const prevAddresses = wallet.walletsAddresses;
    const nextAddresses = Object.keys(_wallets).map((a) => a.toLowerCase()).sort();

    if (isEqual(prevAddresses, nextAddresses)) {
      return;
    }

    if (wallet.filterSubId) {
      api.eth.uninstallFilter(wallet.filterSubId);
    }

    if (nextAddresses.length === 0) {
      return dispatch(updateWallets({ wallets: {}, walletsAddresses: [], filterSubId: null }));
    }

    const filterOptions = {
      fromBlock: 0,
      toBlock: 'latest',
      address: nextAddresses
    };

    api.eth
      .newFilter(filterOptions)
      .then((filterId) => {
        dispatch(updateWallets({ wallets: _wallets, walletsAddresses: nextAddresses, filterSubId: filterId }));
      })
      .catch((error) => {
        if (process.env.NODE_ENV === 'production') {
          console.error('walletActions::attachWallets', error);
        } else {
          throw error;
        }
      });

    fetchWalletsInfo(Object.keys(_wallets))(dispatch, getState);
  };
}

export function load (api) {
  return (dispatch, getState) => {
    const contract = new Contract(api, WALLET_ABI);

    dispatch(setWalletContract(contract));
    api.subscribe('eth_blockNumber', (error) => {
      if (error) {
        if (process.env.NODE_ENV === 'production') {
          return console.error('[eth_blockNumber] walletActions::load', error);
        } else {
          throw error;
        }
      }

      const { filterSubId } = getState().wallet;

      if (!filterSubId) {
        return;
      }

      api.eth
        .getFilterChanges(filterSubId)
        .then((logs) => contract.parseEventLogs(logs))
        .then((logs) => {
          parseLogs(logs)(dispatch, getState);
        })
        .catch((error) => {
          if (process.env.NODE_ENV === 'production') {
            return console.error('[getFilterChanges] walletActions::load', error);
          } else {
            throw error;
          }
        });
    });
  };
}

function fetchWalletsInfo (updates) {
  return (dispatch, getState) => {
    if (Array.isArray(updates)) {
      const _updates = updates.reduce((updates, address) => {
        updates[address] = {
          [ UPDATE_OWNERS ]: true,
          [ UPDATE_REQUIRE ]: true,
          [ UPDATE_DAILYLIMIT ]: true,
          [ UPDATE_CONFIRMATIONS ]: true,
          [ UPDATE_TRANSACTIONS ]: true,
          address
        };

        return updates;
      }, {});

      return fetchWalletsInfo(_updates)(dispatch, getState);
    }

    const { api } = getState();
    const _updates = Object.values(updates);

    Promise
      .all(_updates.map((update) => {
        const contract = new Contract(api, WALLET_ABI).at(update.address);

        return fetchWalletInfo(contract, update, getState);
      }))
      .then((updates) => {
        dispatch(updateWalletsDetails(updates));
      })
      .catch((error) => {
        if (process.env.NODE_ENV === 'production') {
          return console.error('walletAction::fetchWalletsInfo', error);
        } else {
          throw error;
        }
      });
  };
}

function fetchWalletInfo (contract, update, getState) {
  const promises = [];

  if (update[UPDATE_OWNERS]) {
    promises.push(fetchWalletOwners(contract));
  }

  if (update[UPDATE_REQUIRE]) {
    promises.push(fetchWalletRequire(contract));
  }

  if (update[UPDATE_DAILYLIMIT]) {
    promises.push(fetchWalletDailylimit(contract));
  }

  if (update[UPDATE_TRANSACTIONS]) {
    promises.push(fetchWalletTransactions(contract));
  }

  return Promise
    .all(promises)
    .then((updates) => {
      if (update[UPDATE_CONFIRMATIONS]) {
        const ownersUpdate = updates.find((u) => u.key === UPDATE_OWNERS);
        const transactionsUpdate = updates.find((u) => u.key === UPDATE_TRANSACTIONS);

        const owners = ownersUpdate && ownersUpdate.value || null;
        const transactions = transactionsUpdate && transactionsUpdate.value || null;

        return fetchWalletConfirmations(contract, update[UPDATE_CONFIRMATIONS], owners, transactions, getState)
          .then((update) => {
            updates.push(update);
            return updates;
          });
      }

      return updates;
    })
    .then((updates) => {
      const wallet = { address: update.address };

      updates.forEach((update) => {
        wallet[update.key] = update.value;
      });

      return wallet;
    });
}

function fetchWalletTransactions (contract) {
  return WalletsUtils
    .fetchTransactions(contract)
    .then((transactions) => {
      return {
        key: UPDATE_TRANSACTIONS,
        value: transactions
      };
    });
}

function fetchWalletOwners (contract) {
  return WalletsUtils
    .fetchOwners(contract)
    .then((value) => {
      return {
        key: UPDATE_OWNERS,
        value
      };
    });
}

function fetchWalletRequire (contract) {
  return WalletsUtils
    .fetchRequire(contract)
    .then((value) => {
      return {
        key: UPDATE_REQUIRE,
        value
      };
    });
}

function fetchWalletDailylimit (contract) {
  return WalletsUtils
    .fetchDailylimit(contract)
    .then((value) => {
      return {
        key: UPDATE_DAILYLIMIT,
        value
      };
    });
}

function fetchWalletConfirmations (contract, _operations, _owners = null, _transactions = null, getState) {
  const walletInstance = contract.instance;

  const wallet = getState().wallet.wallets[contract.address];

  const owners = _owners || (wallet && wallet.owners) || null;
  const transactions = _transactions || (wallet && wallet.transactions) || null;
  // Full load if no operations given, or if the one given aren't loaded yet
  const fullLoad = !Array.isArray(_operations) || _operations
    .filter((op) => !wallet.confirmations.find((conf) => conf.operation === op))
    .length > 0;

  let promise;

  if (fullLoad) {
    promise = walletInstance
      .ConfirmationNeeded
      .getAllLogs()
      .then((logs) => {
        return logs.map((log) => ({
          initiator: log.params.initiator.value,
          to: log.params.to.value,
          data: log.params.data.value,
          value: log.params.value.value,
          operation: bytesToHex(log.params.operation.value),
          transactionIndex: log.transactionIndex,
          transactionHash: log.transactionHash,
          blockNumber: log.blockNumber,
          confirmedBy: []
        }));
      })
      .then((logs) => {
        return logs.sort((logA, logB) => {
          const comp = logA.blockNumber.comparedTo(logB.blockNumber);

          if (comp !== 0) {
            return comp;
          }

          return logA.transactionIndex.comparedTo(logB.transactionIndex);
        });
      })
      .then((confirmations) => {
        if (confirmations.length === 0) {
          return confirmations;
        }

        // Only fetch confirmations for operations not
        // yet confirmed (ie. not yet a transaction)
        if (transactions) {
          const operations = transactions
            .filter((t) => t.operation)
            .map((t) => t.operation);

          return confirmations.filter((confirmation) => {
            return !operations.includes(confirmation.operation);
          });
        }

        return confirmations;
      });
  } else {
    const { confirmations } = wallet;
    const nextConfirmations = confirmations
      .filter((conf) => _operations.includes(conf.operation));

    promise = Promise.resolve(nextConfirmations);
  }

  return promise
    .then((confirmations) => {
      if (confirmations.length === 0) {
        return confirmations;
      }

      const uniqConfirmations = Object.values(
        confirmations.reduce((confirmations, confirmation) => {
          confirmations[confirmation.operation] = confirmation;
          return confirmations;
        }, {})
      );

      const operations = uniqConfirmations.map((conf) => conf.operation);

      return Promise
        .all(operations.map((op) => fetchOperationConfirmations(contract, op, owners)))
        .then((confirmedBys) => {
          uniqConfirmations.forEach((_, index) => {
            uniqConfirmations[index].confirmedBy = confirmedBys[index];
          });

          return uniqConfirmations;
        });
    })
    .then((confirmations) => {
      const prevConfirmations = wallet.confirmations || [];
      const nextConfirmations = prevConfirmations
        .filter((conA) => !confirmations.find((conB) => conB.operation === conA.operation))
        .concat(confirmations)
        .map((conf) => ({
          ...conf,
          pending: false
        }));

      return {
        key: UPDATE_CONFIRMATIONS,
        value: nextConfirmations
      };
    });
}

function fetchOperationConfirmations (contract, operation, owners = null) {
  if (!owners) {
    console.warn('[fetchOperationConfirmations] try to provide the owners for the Wallet', contract.address);
  }

  const walletInstance = contract.instance;

  const promise = owners
    ? Promise.resolve({ value: owners })
    : fetchWalletOwners(contract);

  return promise
    .then((result) => {
      const owners = result.value;

      return Promise
        .all(owners.map((owner) => walletInstance.hasConfirmed.call({}, [ operation, owner ])))
        .then((data) => {
          return owners.filter((owner, index) => data[index]);
        });
    });
}

function parseLogs (logs) {
  return (dispatch, getState) => {
    if (!logs || logs.length === 0) {
      return;
    }

    const WalletSignatures = WalletsUtils.getWalletSignatures();

    const updates = {};

    logs.forEach((log) => {
      const { address, topics } = log;
      const eventSignature = toHex(topics[0]);
      const prev = updates[address] || {
        [ UPDATE_DAILYLIMIT ]: true,
        address
      };

      switch (eventSignature) {
        case WalletSignatures.OwnerChanged:
        case WalletSignatures.OwnerAdded:
        case WalletSignatures.OwnerRemoved:
          updates[address] = {
            ...prev,
            [ UPDATE_OWNERS ]: true
          };
          return;

        case WalletSignatures.RequirementChanged:
          updates[address] = {
            ...prev,
            [ UPDATE_REQUIRE ]: true
          };
          return;

        case WalletSignatures.ConfirmationNeeded:
        case WalletSignatures.Confirmation:
        case WalletSignatures.Revoke:
          const operation = bytesToHex(log.params.operation.value);

          updates[address] = {
            ...prev,
            [ UPDATE_CONFIRMATIONS ]: uniq(
              (prev[UPDATE_CONFIRMATIONS] || []).concat(operation)
            )
          };

          return;

        case WalletSignatures.Deposit:
        case WalletSignatures.SingleTransact:
        case WalletSignatures.MultiTransact:
        case WalletSignatures.Old.SingleTransact:
        case WalletSignatures.Old.MultiTransact:
          updates[address] = {
            ...prev,
            [ UPDATE_TRANSACTIONS ]: true
          };
          return;
      }
    });

    fetchWalletsInfo(updates)(dispatch, getState);
  };
}

function setOperationPendingState (address, operation, isPending) {
  return {
    type: 'setOperationPendingState',
    address, operation, isPending
  };
}

function updateWalletsDetails (wallets) {
  return {
    type: 'updateWalletsDetails',
    wallets
  };
}

function setWalletContract (contract) {
  return {
    type: 'setWalletContract',
    contract
  };
}

function updateWallets (data) {
  return {
    type: 'updateWallets',
    ...data
  };
}
