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

import { uniq } from 'lodash';
import { push } from 'react-router-redux';

import { notifyTransaction } from '~/util/notifications';
import { ETH_TOKEN, fetchAccountsBalances } from '~/util/tokens';
import { LOG_KEYS, getLogger } from '~/config';
import { sha3 } from '~/api/util/sha3';

const TRANSFER_SIGNATURE = sha3('Transfer(address,address,uint256)');

const log = getLogger(LOG_KEYS.Balances);

let tokensFilter = {
  tokenAddresses: [],
  addresses: []
};

function _setBalances (balances) {
  return {
    type: 'setBalances',
    balances
  };
}

/**
 * @param {Object}  _balances         - In the shape:
 *   {
 *     [ who ]: { [ tokenId ]: BigNumber } // The balances of `who`
 *   }
 * @param {Boolean} skipNotifications [description]
 */
function setBalances (updates, skipNotifications = false) {
  return (dispatch, getState) => {
    const { tokens, balances } = getState();

    const prevBalances = balances;
    const nextBalances = { ...prevBalances };

    Object.keys(updates)
      .forEach((who) => {
        const accountUpdates = updates[who];

        Object.keys(accountUpdates)
          .forEach((tokenId) => {
            const token = tokens[tokenId];
            const prevTokenValue = (prevBalances[who] || {})[tokenId];
            const nextTokenValue = accountUpdates[tokenId];

            if (prevTokenValue && prevTokenValue.lt(nextTokenValue)) {
              dispatch(notifyBalanceChange(who, prevTokenValue, nextTokenValue, token));
            }

            nextBalances[who] = {
              ...(nextBalances[who] || {}),
              [tokenId]: nextTokenValue
            };
          });
      });

    return dispatch(_setBalances(nextBalances));
  };
}

function notifyBalanceChange (who, fromValue, toValue, token) {
  return (dispatch, getState) => {
    const account = getState().personal.accounts[who];

    if (account) {
      const txValue = toValue.minus(fromValue);

      const redirectToAccount = () => {
        const basePath = account.wallet
          ? 'wallet'
          : 'accounts';

        const route = `/${basePath}/${account.address}`;

        dispatch(push(route));
      };

      notifyTransaction(account, token, txValue, redirectToAccount);
    }
  };
}

// TODO: fetch txCount when needed
export function fetchBalances (addresses, skipNotifications = false) {
  return (dispatch, getState) => {
    const { personal } = getState();
    const { visibleAccounts, accounts } = personal;

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const updates = (addresses || addressesToFetch).reduce((updates, who) => {
      updates[who] = [ ETH_TOKEN.id ];

      return updates;
    }, {});

    return fetchTokensBalances(updates, skipNotifications)(dispatch, getState);
  };
}

export function updateTokensFilter (options = {}) {
  return (dispatch, getState) => {
    const { api, personal, tokens } = getState();
    const { visibleAccounts, accounts } = personal;

    const addresses = uniq(visibleAccounts.concat(Object.keys(accounts))).sort();
    const tokensToUpdate = Object.values(tokens);
    const tokensAddressMap = Object.values(tokens).reduce((map, token) => {
      map[token.address] = token;
      return map;
    }, {});

    const tokenAddresses = tokensToUpdate
      .map((t) => t.address)
      .filter((address) => address)
      .sort();

    // Token Addresses that are not in the current filter
    const newTokenAddresses = tokenAddresses
      .filter((tokenAddress) => !tokensFilter.tokenAddresses.includes(tokenAddress));

    // Addresses that are not in the current filter (omit those
    // that the filter includes)
    const newAddresses = addresses
      .filter((address) => !tokensFilter.addresses.includes(address));

    if (tokensFilter.filterFromId || tokensFilter.filterToId) {
      // If no new addresses and the same tokens, don't change the filter
      if (newTokenAddresses.length === 0 && newAddresses.length === 0) {
        log.debug('no need to update token filter', addresses, tokenAddresses, tokensFilter);
        return;
      }
    }

    const promises = [];
    const updates = {};

    // Fetch all tokens for each new address
    newAddresses.forEach((address) => {
      updates[address] = tokensToUpdate.map((token) => token.id);
    });

    // Fetch new token balances for each address
    newTokenAddresses.forEach((tokenAddress) => {
      const token = tokensAddressMap[tokenAddress];

      addresses.forEach((address) => {
        updates[address] = uniq((updates[address] || []).concat(token.id));
      });
    });

    log.debug('updating the token filter', addresses, tokenAddresses);

    const topicsFrom = [ TRANSFER_SIGNATURE, addresses, null ];
    const topicsTo = [ TRANSFER_SIGNATURE, null, addresses ];

    const filterOptions = {
      fromBlock: 'latest',
      toBlock: 'latest',
      address: tokenAddresses
    };

    const optionsFrom = {
      ...filterOptions,
      topics: topicsFrom
    };

    const optionsTo = {
      ...filterOptions,
      topics: topicsTo
    };

    promises.push(
      api.eth.newFilter(optionsFrom),
      api.eth.newFilter(optionsTo)
    );

    if (tokensFilter.filterFromId) {
      promises.push(api.eth.uninstallFilter(tokensFilter.filterFromId));
    }

    if (tokensFilter.filterToId) {
      promises.push(api.eth.uninstallFilter(tokensFilter.filterToId));
    }

    return Promise.all(promises)
      .then(([ filterFromId, filterToId ]) => {
        const nextTokensFilter = {
          filterFromId, filterToId,
          addresses, tokenAddresses
        };

        tokensFilter = nextTokensFilter;
      })
      .then(() => fetchTokensBalances(updates)(dispatch, getState))
      .catch((error) => {
        console.warn('balances::updateTokensFilter', error);
      });
  };
}

export function queryTokensFilter () {
  return (dispatch, getState) => {
    const { api } = getState();

    Promise
      .all([
        api.eth.getFilterChanges(tokensFilter.filterFromId),
        api.eth.getFilterChanges(tokensFilter.filterToId)
      ])
      .then(([ logsFrom, logsTo ]) => {
        const logs = [].concat(logsFrom, logsTo);

        if (logs.length === 0) {
          return;
        } else {
          log.debug('got tokens filter logs', logs);
        }

        const { personal, tokens } = getState();
        const { visibleAccounts, accounts } = personal;

        const allAddresses = visibleAccounts.concat(Object.keys(accounts));
        const addressesToFetch = uniq(allAddresses);
        const lcAddresses = addressesToFetch.map((a) => a.toLowerCase());

        const lcTokensMap = Object.values(tokens).reduce((map, token) => {
          map[token.address.toLowerCase()] = token;
          return map;
        });

        // The keys are the account addresses,
        // and the value is an Array of the tokens addresses
        // to update
        const updates = {};

        logs
          .forEach((log, index) => {
            const tokenAddress = log.address.toLowerCase();
            const token = lcTokensMap[tokenAddress];

            // logs = [ ...logsFrom, ...logsTo ]
            if (index < logsFrom.length) {
              const fromAddress = ('0x' + log.topics[1].slice(-40)).toLowerCase();
              const fromAddressIndex = lcAddresses.indexOf(fromAddress);

              if (fromAddressIndex > -1) {
                const who = addressesToFetch[fromAddressIndex];

                updates[who] = [].concat(updates[who] || [], token.id);
              }
            } else {
              const toAddress = ('0x' + log.topics[2].slice(-40)).toLowerCase();
              const toAddressIndex = lcAddresses.indexOf(toAddress);

              if (toAddressIndex > -1) {
                const who = addressesToFetch[toAddressIndex];

                updates[who] = [].concat(updates[who] || [], token.id);
              }
            }
          });

        // No accounts to update
        if (Object.keys(updates).length === 0) {
          return;
        }

        Object.keys(updates).forEach((who) => {
          // Keep non-empty token addresses
          updates[who] = uniq(updates[who]);
        });

        fetchTokensBalances(updates)(dispatch, getState);
      });
  };
}

export function fetchTokensBalances (updates, skipNotifications = false) {
  return (dispatch, getState) => {
    const { api, personal, tokens } = getState();
    const allTokens = Object.values(tokens);

    if (!updates) {
      const { visibleAccounts, accounts } = personal;
      const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));

      updates = addressesToFetch.reduce((updates, who) => {
        updates[who] = allTokens.map((token) => token.id);

        return updates;
      }, {});
    }

    return fetchAccountsBalances(api, allTokens, updates)
      .then((balances) => {
        log.debug('got tokens balances', balances, updates);
        dispatch(setBalances(balances, skipNotifications));
      })
      .catch((error) => {
        console.warn('balances::fetchTokensBalances', error);
      });
  };
}
