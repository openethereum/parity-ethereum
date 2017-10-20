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

import { uniq, isEqual } from 'lodash';
import { push } from 'react-router-redux';

import { notifyTransaction } from '~/util/notifications';
import { ETH_TOKEN, fetchAccountsBalances } from '~/util/tokens';
import { LOG_KEYS, getLogger } from '~/config';
import { sha3 } from '~/api/util/sha3';

const TRANSFER_SIGNATURE = sha3('Transfer(address,address,uint256)');

const log = getLogger(LOG_KEYS.Balances);

let tokensFilter = {};

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

            // Add the token if it's native ETH or if it has a value
            if (token.native || nextTokenValue.gt(0)) {
              nextBalances[who] = {
                ...(nextBalances[who] || {}),
                [tokenId]: nextTokenValue
              };
            }
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
export function fetchBalances (_addresses, skipNotifications = false) {
  return fetchTokensBalances(_addresses, [ ETH_TOKEN ], skipNotifications);
}

export function updateTokensFilter (_addresses, _tokens, options = {}) {
  return (dispatch, getState) => {
    const { api, personal, tokens } = getState();
    const { visibleAccounts, accounts } = personal;

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = uniq(_addresses || addressesToFetch || []).sort();

    const tokensToUpdate = _tokens || Object.values(tokens);
    const tokenAddresses = tokensToUpdate
      .map((t) => t.address)
      .filter((address) => address)
      .sort();

    if (tokensFilter.filterFromId || tokensFilter.filterToId) {
      // Has the tokens addresses changed (eg. a network change)
      const sameTokens = isEqual(tokenAddresses, tokensFilter.tokenAddresses);

      // Addresses that are not in the current filter (omit those
      // that the filter includes)
      const newAddresses = addresses.filter((address) => !tokensFilter.addresses.includes(address));

      // If no new addresses and the same tokens, don't change the filter
      if (sameTokens && newAddresses.length === 0) {
        log.debug('no need to update token filter', addresses, tokenAddresses, tokensFilter);
        return queryTokensFilter(tokensFilter)(dispatch, getState);
      }
    }

    log.debug('updating the token filter', addresses, tokenAddresses);
    const promises = [];

    if (tokensFilter.filterFromId) {
      promises.push(api.eth.uninstallFilter(tokensFilter.filterFromId));
    }

    if (tokensFilter.filterToId) {
      promises.push(api.eth.uninstallFilter(tokensFilter.filterToId));
    }

    Promise
      .all([
        api.eth.blockNumber()
      ].concat(promises))
      .then(([ block ]) => {
        const topicsFrom = [ TRANSFER_SIGNATURE, addresses, null ];
        const topicsTo = [ TRANSFER_SIGNATURE, null, addresses ];

        const filterOptions = {
          fromBlock: block,
          toBlock: 'pending',
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

        const newFilters = Promise.all([
          api.eth.newFilter(optionsFrom),
          api.eth.newFilter(optionsTo)
        ]);

        return newFilters;
      })
      .then(([ filterFromId, filterToId ]) => {
        const nextTokensFilter = {
          filterFromId, filterToId,
          addresses, tokenAddresses
        };

        const { skipNotifications } = options;

        tokensFilter = nextTokensFilter;
        fetchTokensBalances(addresses, tokensToUpdate, skipNotifications)(dispatch, getState);
      })
      .catch((error) => {
        console.warn('balances::updateTokensFilter', error);
      });
  };
}

export function queryTokensFilter () {
  return (dispatch, getState) => {
    const { api, personal, tokens } = getState();
    const { visibleAccounts, accounts } = personal;

    const allAddresses = visibleAccounts.concat(Object.keys(accounts));
    const addressesToFetch = uniq(allAddresses);
    const lcAddresses = addressesToFetch.map((a) => a.toLowerCase());

    Promise
      .all([
        api.eth.getFilterChanges(tokensFilter.filterFromId),
        api.eth.getFilterChanges(tokensFilter.filterToId)
      ])
      .then(([ logsFrom, logsTo ]) => {
        const addresses = [];
        const tokenAddresses = [];
        const logs = logsFrom.concat(logsTo);

        if (logs.length > 0) {
          log.debug('got tokens filter logs', logs);
        }

        logs
          .forEach((log) => {
            const tokenAddress = log.address;

            const fromAddress = '0x' + log.topics[1].slice(-40);
            const toAddress = '0x' + log.topics[2].slice(-40);

            const fromAddressIndex = lcAddresses.indexOf(fromAddress);
            const toAddressIndex = lcAddresses.indexOf(toAddress);

            if (fromAddressIndex > -1) {
              addresses.push(addressesToFetch[fromAddressIndex]);
            }

            if (toAddressIndex > -1) {
              addresses.push(addressesToFetch[toAddressIndex]);
            }

            tokenAddresses.push(tokenAddress);
          });

        if (addresses.length === 0) {
          return;
        }

        const tokensToUpdate = Object.values(tokens)
          .filter((t) => tokenAddresses.includes(t.address));

        fetchTokensBalances(uniq(addresses), tokensToUpdate)(dispatch, getState);
      });
  };
}

export function fetchTokensBalances (_addresses = null, _tokens = null, skipNotifications = false) {
  return (dispatch, getState) => {
    const { api, personal, tokens } = getState();
    const { visibleAccounts, accounts } = personal;
    const allTokens = Object.values(tokens);

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = _addresses || addressesToFetch;
    const tokensToUpdate = _tokens || allTokens;

    if (addresses.length === 0) {
      return Promise.resolve();
    }

    const updates = addresses.reduce((updates, who) => {
      updates[who] = tokensToUpdate.map((token) => token.id);
      return updates;
    }, {});

    return fetchAccountsBalances(api, allTokens, updates)
      .then((balances) => {
        dispatch(setBalances(balances, skipNotifications));
      })
      .catch((error) => {
        console.warn('balances::fetchTokensBalances', error);
      });
  };
}
