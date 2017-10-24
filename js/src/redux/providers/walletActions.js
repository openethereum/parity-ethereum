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
import { ERROR_CODES } from '~/api/transport/error';
import { foundationWallet as WALLET_ABI } from '~/contracts/abi';
import WalletsUtils from '~/util/wallets';
import { newError } from '~/ui/Errors/actions';
import {
  UPDATE_OWNERS,
  UPDATE_REQUIRE,
  UPDATE_DAILYLIMIT,
  UPDATE_TRANSACTIONS,
  UPDATE_CONFIRMATIONS
} from '~/util/wallets/updates';

export function confirmOperation (address, owner, operation) {
  return modifyOperation('confirm', address, owner, operation);
}

export function revokeOperation (address, owner, operation) {
  return modifyOperation('revoke', address, owner, operation);
}

function modifyOperation (modification, address, owner, operation) {
  return (dispatch, getState) => {
    const { api } = getState();

    dispatch(setOperationPendingState(address, operation, true));

    WalletsUtils.postModifyOperation(api, address, modification, owner, operation)
      .then((requestId) => {
        return api.pollMethod('parity_checkRequest', requestId);
      })
      .catch((error) => {
        if (error.code === ERROR_CODES.REQUEST_REJECTED) {
          return;
        }

        dispatch(newError(error));
      })
      .then(() => {
        dispatch(setOperationPendingState(address, operation, false));
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

    // Filter the logs from the current block
    api.eth
      .blockNumber()
      .then((block) => {
        const filterOptions = {
          fromBlock: block,
          toBlock: 'latest',
          address: nextAddresses
        };

        return api.eth.newFilter(filterOptions);
      })
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
  const wallet = getState().wallet.wallets[contract.address];

  const owners = _owners || (wallet && wallet.owners) || null;
  const transactions = _transactions || (wallet && wallet.transactions) || null;
  const cache = { owners, transactions };

  return WalletsUtils.fetchPendingTransactions(contract, cache)
    .then((confirmations) => {
      return {
        key: UPDATE_CONFIRMATIONS,
        value: confirmations
      };
    });
}

function parseLogs (logs) {
  return (dispatch, getState) => {
    if (!logs || logs.length === 0) {
      return;
    }

    const { api } = getState();
    const updates = {};

    const promises = logs.map((log) => {
      const { address } = log;

      return WalletsUtils.logToUpdate(api, address, log)
        .then((update) => {
          const prev = updates[address] || {
            [ UPDATE_DAILYLIMIT ]: true,
            address
          };

          if (update[UPDATE_CONFIRMATIONS]) {
            const operations = (prev[UPDATE_CONFIRMATIONS] || []).concat(update[UPDATE_CONFIRMATIONS]);

            update[UPDATE_CONFIRMATIONS] = uniq(operations);
          }

          updates[address] = {
            ...prev,
            ...update
          };
        });
    });

    return Promise.all(promises)
      .then(() => {
        fetchWalletsInfo(updates)(dispatch, getState);
      });
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
