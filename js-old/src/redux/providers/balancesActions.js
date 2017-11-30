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

import { difference, uniq } from 'lodash';
import { push } from 'react-router-redux';

import { notifyTransaction } from '~/util/notifications';
import { ETH_TOKEN, fetchAccountsBalances } from '~/util/tokens';
import { LOG_KEYS, getLogger } from '~/config';
import { sha3 } from '@parity/api/lib/util/sha3';

import { fetchTokens } from './tokensActions';

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

    const addressesToFetch = addresses || uniq(visibleAccounts.concat(Object.keys(accounts)));
    const updates = addressesToFetch.reduce((updates, who) => {
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

    const addresses = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const tokensToUpdate = Object.values(tokens);
    const tokensAddressMap = Object.values(tokens).reduce((map, token) => {
      map[token.address] = token;
      return map;
    }, {});

    const tokenAddresses = tokensToUpdate
      .map((t) => t.address)
      .filter((address) => address && !/^(0x)?0*$/.test(address));

    // Token Addresses that are not in the current filter
    const newTokenAddresses = difference(tokenAddresses, tokensFilter.tokenAddresses);

    // Addresses that are not in the current filter (omit those
    // that the filter includes)
    const newAddresses = difference(addresses, tokensFilter.addresses);

    if (tokensFilter.filterFromId || tokensFilter.filterToId) {
      // If no new addresses and the same tokens, don't change the filter
      if (newTokenAddresses.length === 0 && newAddresses.length === 0) {
        log.debug('no need to update token filter', addresses, tokenAddresses, tokensFilter);
        return;
      }
    }

    const promises = [];
    const updates = {};

    const allTokenIds = tokensToUpdate.map((token) => token.id);
    const newTokenIds = newTokenAddresses.map((address) => tokensAddressMap[address].id);

    newAddresses.forEach((newAddress) => {
      updates[newAddress] = allTokenIds;
    });

    difference(addresses, newAddresses).forEach((oldAddress) => {
      updates[oldAddress] = newTokenIds;
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

        const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
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
            const topicIdx = index < logsFrom.length ? 1 : 2;
            const address = ('0x' + log.topics[topicIdx].slice(-40)).toLowerCase();
            const addressIndex = lcAddresses.indexOf(address);

            if (addressIndex > -1) {
              const who = addressesToFetch[addressIndex];

              updates[who] = [].concat(updates[who] || [], token.id);
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

    let start = Date.now();

    return fetchAccountsBalances(api, allTokens, updates)
      .then((balances) => {
        log.debug('got tokens balances', balances, updates, `(took ${Date.now() - start}ms)`);

        // Tokens info might not be fetched yet (to not load
        // tokens we don't care about)
        const tokenIdsToFetch = Object.values(balances)
          .reduce((tokenIds, balance) => {
            const nextTokenIds = Object.keys(balance)
              .filter((tokenId) => balance[tokenId].gt(0));

            return tokenIds.concat(nextTokenIds);
          }, []);

        const tokenIndexesToFetch = uniq(tokenIdsToFetch)
          .filter((tokenId) => tokens[tokenId] && tokens[tokenId].index && !tokens[tokenId].fetched)
          .map((tokenId) => tokens[tokenId].index);

        if (tokenIndexesToFetch.length === 0) {
          return balances;
        }

        start = Date.now();
        return fetchTokens(tokenIndexesToFetch)(dispatch, getState)
          .then(() => log.debug('token indexes fetched', tokenIndexesToFetch, `(took ${Date.now() - start}ms)`))
          .then(() => balances);
      })
      .then((balances) => {
        dispatch(setBalances(balances, skipNotifications));
      })
      .catch((error) => {
        console.warn('balances::fetchTokensBalances', error);
      });
  };
}
