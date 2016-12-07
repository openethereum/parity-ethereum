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

import { isEqual, uniq, range } from 'lodash';

import Contract from '../../api/contract';
import { wallet as WALLET_ABI } from '../../contracts/abi';
import { bytesToHex, toHex } from '../../api/util/format';

import { ERROR_CODES } from '../../api/transport/error';
import { MAX_GAS_ESTIMATION } from '../../util/constants';

import { newError } from '../../ui/Errors/actions';

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
        options.gas = gas;
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

        return fetchWalletConfirmations(contract, owners, transactions, getState)
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
  const walletInstance = contract.instance;
  const signatures = {
    single: toHex(walletInstance.SingleTransact.signature),
    multi: toHex(walletInstance.MultiTransact.signature),
    deposit: toHex(walletInstance.Deposit.signature)
  };

  return contract
    .getAllLogs({
      topics: [ [ signatures.single, signatures.multi, signatures.deposit ] ]
    })
    .then((logs) => {
      return logs.sort((logA, logB) => {
        const comp = logB.blockNumber.comparedTo(logA.blockNumber);

        if (comp !== 0) {
          return comp;
        }

        return logB.transactionIndex.comparedTo(logA.transactionIndex);
      });
    })
    .then((logs) => {
      const transactions = logs.map((log) => {
        const signature = toHex(log.topics[0]);

        const value = log.params.value.value;
        const from = signature === signatures.deposit
          ? log.params['_from'].value
          : contract.address;

        const to = signature === signatures.deposit
          ? contract.address
          : log.params.to.value;

        const transaction = {
          transactionHash: log.transactionHash,
          blockNumber: log.blockNumber,
          from, to, value
        };

        if (log.params.operation) {
          transaction.operation = bytesToHex(log.params.operation.value);
        }

        if (log.params.data) {
          transaction.data = log.params.data.value;
        }

        return transaction;
      });

      return {
        key: UPDATE_TRANSACTIONS,
        value: transactions
      };
    });
}

function fetchWalletOwners (contract) {
  const walletInstance = contract.instance;

  return walletInstance
    .m_numOwners.call()
    .then((mNumOwners) => {
      return Promise.all(range(mNumOwners.toNumber()).map((idx) => walletInstance.getOwner.call({}, [ idx ])));
    })
    .then((value) => {
      return {
        key: UPDATE_OWNERS,
        value
      };
    });
}

function fetchWalletRequire (contract) {
  const walletInstance = contract.instance;

  return walletInstance
    .m_required.call()
    .then((value) => {
      return {
        key: UPDATE_REQUIRE,
        value
      };
    });
}

function fetchWalletDailylimit (contract) {
  const walletInstance = contract.instance;

  return Promise
    .all([
      walletInstance.m_dailyLimit.call(),
      walletInstance.m_spentToday.call(),
      walletInstance.m_lastDay.call()
    ])
    .then((values) => {
      return {
        key: UPDATE_DAILYLIMIT,
        value: {
          limit: values[0],
          spent: values[1],
          last: values[2]
        }
      };
    });
}

function fetchWalletConfirmations (contract, _owners = null, _transactions = null, getState) {
  const walletInstance = contract.instance;

  const wallet = getState().wallet.wallets[contract.address];

  const owners = _owners || (wallet && wallet.owners) || null;
  const transactions = _transactions || (wallet && wallet.transactions) || null;

  return walletInstance
    .ConfirmationNeeded
    .getAllLogs()
    .then((logs) => {
      return logs.sort((logA, logB) => {
        const comp = logA.blockNumber.comparedTo(logB.blockNumber);

        if (comp !== 0) {
          return comp;
        }

        return logA.transactionIndex.comparedTo(logB.transactionIndex);
      });
    })
    .then((logs) => {
      return logs.map((log) => ({
        initiator: log.params.initiator.value,
        to: log.params.to.value,
        data: log.params.data.value,
        value: log.params.value.value,
        operation: bytesToHex(log.params.operation.value),
        transactionHash: log.transactionHash,
        blockNumber: log.blockNumber,
        confirmedBy: []
      }));
    })
    .then((confirmations) => {
      if (confirmations.length === 0) {
        return confirmations;
      }

      if (transactions) {
        const operations = transactions
          .filter((t) => t.operation)
          .map((t) => t.operation);

        return confirmations.filter((confirmation) => {
          return !operations.includes(confirmation.operation);
        });
      }

      return confirmations;
    })
    .then((confirmations) => {
      if (confirmations.length === 0) {
        return confirmations;
      }

      const operations = confirmations.map((conf) => conf.operation);
      return Promise
        .all(operations.map((op) => fetchOperationConfirmations(contract, op, owners)))
        .then((confirmedBys) => {
          confirmations.forEach((_, index) => {
            confirmations[index].confirmedBy = confirmedBys[index];
          });

          return confirmations;
        });
    })
    .then((confirmations) => {
      return {
        key: UPDATE_CONFIRMATIONS,
        value: confirmations
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

    const { wallet } = getState();
    const { contract } = wallet;
    const walletInstance = contract.instance;

    const signatures = {
      OwnerChanged: toHex(walletInstance.OwnerChanged.signature),
      OwnerAdded: toHex(walletInstance.OwnerAdded.signature),
      OwnerRemoved: toHex(walletInstance.OwnerRemoved.signature),
      RequirementChanged: toHex(walletInstance.RequirementChanged.signature),
      Confirmation: toHex(walletInstance.Confirmation.signature),
      Revoke: toHex(walletInstance.Revoke.signature),
      Deposit: toHex(walletInstance.Deposit.signature),
      SingleTransact: toHex(walletInstance.SingleTransact.signature),
      MultiTransact: toHex(walletInstance.MultiTransact.signature),
      ConfirmationNeeded: toHex(walletInstance.ConfirmationNeeded.signature)
    };

    const updates = {};

    logs.forEach((log) => {
      const { address, topics } = log;
      const eventSignature = toHex(topics[0]);
      const prev = updates[address] || { address };

      switch (eventSignature) {
        case signatures.OwnerChanged:
        case signatures.OwnerAdded:
        case signatures.OwnerRemoved:
          updates[address] = {
            ...prev,
            [ UPDATE_OWNERS ]: true
          };
          return;

        case signatures.RequirementChanged:
          updates[address] = {
            ...prev,
            [ UPDATE_REQUIRE ]: true
          };
          return;

        case signatures.Confirmation:
        case signatures.Revoke:
          const operation = log.params.operation.value;

          updates[address] = {
            ...prev,
            [ UPDATE_CONFIRMATIONS ]: uniq(
              (prev.operations || []).concat(operation)
            )
          };
          return;

        case signatures.Deposit:
        case signatures.SingleTransact:
        case signatures.MultiTransact:
          updates[address] = {
            ...prev,
            [ UPDATE_TRANSACTIONS ]: true
          };
          return;

        case signatures.ConfirmationNeeded:
          const op = log.params.operation.value;

          updates[address] = {
            ...prev,
            [ UPDATE_CONFIRMATIONS ]: uniq(
              (prev.operations || []).concat(op)
            )
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
