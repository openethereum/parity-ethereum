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

import { setAddressImage } from './imagesActions';

import { notifyTransaction } from '~/util/notifications';
import { ETH_TOKEN, fetchTokenIds, fetchTokenInfo, fetchAccountsBalances } from '~/util/tokens';
import { LOG_KEYS, getLogger } from '~/config';
import { sha3 } from '~/api/util/sha3';

const TRANSFER_SIGNATURE = sha3('Transfer(address,address,uint256)');

const log = getLogger(LOG_KEYS.Balances);

/**
 * @param {Object}  _balances         - In the shape:
 *   {
 *     [ who ]: { [ tokenId ]: BigNumber } // The balances of `who`
 *   }
 * @param {Boolean} skipNotifications [description]
 */
function setBalances (updates, skipNotifications = false) {
  return (dispatch, getState) => {
    const state = getState();
    const tokens = Object.values(state.balances.tokens);

    const prevBalances = state.balances.balances;
    const nextBalances = { ...prevBalances };

    Object.keys(updates).forEach((who) => {
      const accountUpdates = updates[who];

      const prevAccountTokens = (prevBalances[who] || {}).tokens || [];
      const nextAccountTokens = prevAccountTokens.slice();

      const prevAccountBalancesTokenIds = prevAccountTokens.map((tok) => tok.token.id);
      const nextAccountBalancesTokenIds = Object.keys(accountUpdates);

      const existingTokens = nextAccountBalancesTokenIds.filter((id) => prevAccountBalancesTokenIds.includes(id));
      const newTokens = nextAccountBalancesTokenIds.filter((id) => !prevAccountBalancesTokenIds.includes(id));

      existingTokens.forEach((tokenId) => {
        const token = tokens.find((token) => token.id === tokenId);
        const tokenIndex = nextAccountTokens.findIndex((tok) => tok.token.id === tokenId);
        const prevValue = nextAccountTokens[tokenIndex].value;
        const nextValue = accountUpdates[tokenId];

        // If received a token/eth (old value < new value), notify
        if (prevValue.lt(nextValue) && !skipNotifications) {
          dispatch(notifyBalanceChange(who, prevValue, nextValue, token));
        }

        nextAccountTokens[tokenIndex].value = nextValue;
      });

      newTokens.forEach((tokenId) => {
        const token = tokens.find((tok) => tok.id.toString() === tokenId.toString());
        const value = accountUpdates[tokenId];

        // Add the token if it's native ETH or if it has a value
        if (token.native || value.gt(0)) {
          nextAccountTokens.push({ token, value });
        }
      });

      nextBalances[who] = {
        ...(nextBalances[who] || {}),
        tokens: nextAccountTokens
      };
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

function _setBalances (balances) {
  return {
    type: 'setBalances',
    balances
  };
}

export function setTokens (tokens) {
  return {
    type: 'setTokens',
    tokens
  };
}

export function setTokenReg (tokenreg) {
  return {
    type: 'setTokenReg',
    tokenreg
  };
}

export function setTokensFilter (tokensFilter) {
  return {
    type: 'setTokensFilter',
    tokensFilter
  };
}

export function setTokenImage (tokenAddress, image) {
  return {
    type: 'setTokenImage',
    tokenAddress, image
  };
}

export function loadTokens (options = {}) {
  log.debug('loading tokens', Object.keys(options).length ? options : '');

  return (dispatch, getState) => {
    const { tokenreg } = getState().balances;

    return fetchTokenIds(tokenreg.instance)
      .then((tokenIds) => dispatch(fetchTokens(tokenIds, options)))
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  };
}

export function fetchTokens (_tokenIndexes, options = {}) {
  const tokenIndexes = uniq(_tokenIndexes || []);

  return (dispatch, getState) => {
    const { api, images, balances } = getState();
    const { tokenreg } = balances;
    const promises = tokenIndexes.map((id) => fetchTokenInfo(api, tokenreg.instance, id));

    return Promise
      .all(promises)
      .then((tokens) => {
        // dispatch only the changed images
        tokens
          .forEach((token) => {
            const { image, address } = token;

            if (images[address] === image) {
              return;
            }

            dispatch(setTokenImage(address, image));
            dispatch(setAddressImage(address, image, true));
          });

        log.debug('fetched token', tokens);
        dispatch(setTokens(tokens));
        dispatch(updateTokensFilter(null, null, options));
      })
      .catch((error) => {
        console.warn('balances::fetchTokens', error);
      });
  };
}

// TODO: fetch txCount when needed
export function fetchBalances (_addresses, skipNotifications = false) {
  return fetchTokensBalances(_addresses, [ ETH_TOKEN ], skipNotifications);
}

export function updateTokensFilter (_addresses, _tokens, options = {}) {
  return (dispatch, getState) => {
    const { api, balances, personal } = getState();
    const { visibleAccounts, accounts } = personal;
    const { tokensFilter } = balances;

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = uniq(_addresses || addressesToFetch || []).sort();

    const tokens = _tokens || Object.values(balances.tokens) || [];
    const tokenAddresses = tokens
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

    const promise = Promise.all(promises);

    const topicsFrom = [ TRANSFER_SIGNATURE, addresses, null ];
    const topicsTo = [ TRANSFER_SIGNATURE, null, addresses ];

    const options = {
      fromBlock: 0,
      toBlock: 'pending',
      address: tokenAddresses
    };

    const optionsFrom = {
      ...options,
      topics: topicsFrom
    };

    const optionsTo = {
      ...options,
      topics: topicsTo
    };

    const newFilters = Promise.all([
      api.eth.newFilter(optionsFrom),
      api.eth.newFilter(optionsTo)
    ]);

    promise
      .then(() => newFilters)
      .then(([ filterFromId, filterToId ]) => {
        const nextTokensFilter = {
          filterFromId, filterToId,
          addresses, tokenAddresses
        };

        const { skipNotifications } = options;

        dispatch(setTokensFilter(nextTokensFilter));
        fetchTokensBalances(addresses, tokens, skipNotifications)(dispatch, getState);
      })
      .catch((error) => {
        console.warn('balances::updateTokensFilter', error);
      });
  };
}

export function queryTokensFilter (tokensFilter) {
  return (dispatch, getState) => {
    const { api, personal, balances } = getState();
    const { visibleAccounts, accounts } = personal;

    const visibleAddresses = visibleAccounts.map((a) => a.toLowerCase());
    const addressesToFetch = uniq(visibleAddresses.concat(Object.keys(accounts)));

    Promise
      .all([
        api.eth.getFilterChanges(tokensFilter.filterFromId),
        api.eth.getFilterChanges(tokensFilter.filterToId)
      ])
      .then(([ logsFrom, logsTo ]) => {
        const addresses = [];
        const tokenAddresses = [];

        logsFrom
          .concat(logsTo)
          .forEach((log) => {
            const tokenAddress = log.address;

            const fromAddress = '0x' + log.topics[1].slice(-40);
            const toAddress = '0x' + log.topics[2].slice(-40);

            if (addressesToFetch.includes(fromAddress)) {
              addresses.push(fromAddress);
            }

            if (addressesToFetch.includes(toAddress)) {
              addresses.push(toAddress);
            }

            tokenAddresses.push(tokenAddress);
          });

        if (addresses.length === 0) {
          return;
        }

        const tokens = Object.values(balances.tokens)
          .filter((t) => tokenAddresses.includes(t.address));

        fetchTokensBalances(uniq(addresses), tokens)(dispatch, getState);
      });
  };
}

export function fetchTokensBalances (_addresses = null, _tokens = null, skipNotifications = false) {
  return (dispatch, getState) => {
    const { api, personal, balances } = getState();
    const { visibleAccounts, accounts } = personal;
    const allTokens = Object.values(balances.tokens);

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = _addresses || addressesToFetch;
    const tokens = _tokens || allTokens;

    if (addresses.length === 0) {
      return Promise.resolve();
    }

    const updates = addresses.reduce((updates, who) => {
      updates[who] = tokens.map((token) => token.id);
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
