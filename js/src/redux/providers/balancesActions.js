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

import { range, uniq, isEqual } from 'lodash';
import BigNumber from 'bignumber.js';
import { push } from 'react-router-redux';

import { hashToImageUrl } from './imagesReducer';
import { setAddressImage } from './imagesActions';

import * as ABIS from '~/contracts/abi';
import { notifyTransaction } from '~/util/notifications';
import { LOG_KEYS, getLogger } from '~/config';
import imagesEthereum from '~/../assets/images/contracts/ethereum-black-64x64.png';

const log = getLogger(LOG_KEYS.Balances);

const ETH = {
  name: 'Ethereum',
  tag: 'ETH',
  image: imagesEthereum,
  native: true
};

function setBalances (_balances, skipNotifications = false) {
  return (dispatch, getState) => {
    const state = getState();

    const currentTokens = Object.values(state.balances.tokens || {});
    const tokensAddresses = currentTokens
      .map((token) => token.address)
      .filter((address) => address);

    const accounts = state.personal.accounts;
    const nextBalances = _balances;
    const prevBalances = state.balances.balances;
    const balances = { ...prevBalances };

    Object.keys(nextBalances).forEach((address) => {
      if (!balances[address]) {
        balances[address] = Object.assign({}, nextBalances[address]);
        return;
      }

      const balance = Object.assign({}, balances[address]);
      const { tokens, txCount = balance.txCount } = nextBalances[address];

      const prevTokens = balance.tokens.slice();
      const nextTokens = [];

      const handleToken = (prevToken, nextToken) => {
        // If the given token is not in the current tokens, skip
        if (!nextToken && !prevToken) {
          return false;
        }

        // No updates
        if (!nextToken) {
          return nextTokens.push(prevToken);
        }

        const { token, value } = nextToken;

        // If it's a new token, push it
        if (!prevToken) {
          return nextTokens.push({
            token, value
          });
        }

        // Otherwise, update the value
        const prevValue = prevToken.value;

        // If received a token/eth (old value < new value), notify
        if (prevValue.lt(value) && accounts[address] && !skipNotifications) {
          const account = accounts[address];
          const txValue = value.minus(prevValue);

          const redirectToAccount = () => {
            const basePath = account.wallet
              ? 'wallet'
              : 'accounts';

            const route = `/${basePath}/${account.address}`;

            dispatch(push(route));
          };

          notifyTransaction(account, token, txValue, redirectToAccount);
        }

        return nextTokens.push({
          ...prevToken,
          value
        });
      };

      const prevEthToken = prevTokens.find((tok) => tok.token.native);
      const nextEthToken = tokens.find((tok) => tok.token.native);

      handleToken(prevEthToken, nextEthToken);

      tokensAddresses
        .forEach((address) => {
          const prevToken = prevTokens.find((tok) => tok.token.address === address);
          const nextToken = tokens.find((tok) => tok.token.address === address);

          handleToken(prevToken, nextToken);
        });

      balances[address] = { txCount: txCount || new BigNumber(0), tokens: nextTokens };
    });

    dispatch(_setBalances(balances));
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

    return tokenreg.instance.tokenCount
      .call()
      .then((numTokens) => {
        const tokenIds = range(numTokens.toNumber());

        dispatch(fetchTokens(tokenIds, options));
      })
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  };
}

export function fetchTokens (_tokenIds, options = {}) {
  const tokenIds = uniq(_tokenIds || []);

  return (dispatch, getState) => {
    const { api, images, balances } = getState();
    const { tokenreg } = balances;

    return Promise
      .all(tokenIds.map((id) => fetchTokenInfo(tokenreg, id, api)))
      // FIXME ; shouldn't have to filter out tokens...
      .then((tokens) => tokens.filter((token) => token.tag && token.tag.toLowerCase() !== 'eth'))
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

export function fetchBalances (_addresses, skipNotifications = false) {
  return (dispatch, getState) => {
    const { api, personal } = getState();
    const { visibleAccounts, accounts } = personal;

    const addresses = uniq(_addresses || visibleAccounts || []);

    // With only a single account, more info will be displayed.
    const fullFetch = addresses.length === 1;

    // Add accounts addresses (for notifications, accounts selection, etc.)
    const addressesToFetch = uniq(addresses.concat(Object.keys(accounts)));

    return Promise
      .all(addressesToFetch.map((addr) => fetchAccount(addr, api, fullFetch)))
      .then((accountsBalances) => {
        const balances = {};

        addressesToFetch.forEach((addr, idx) => {
          balances[addr] = accountsBalances[idx];
        });

        dispatch(setBalances(balances, skipNotifications));
      })
      .catch((error) => {
        console.warn('balances::fetchBalances', error);
      });
  };
}

export function updateTokensFilter (_addresses, _tokens, options = {}) {
  return (dispatch, getState) => {
    const { api, balances, personal } = getState();
    const { visibleAccounts, accounts } = personal;
    const { tokensFilter } = balances;

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = uniq(_addresses || addressesToFetch || []).sort();

    const tokens = _tokens || Object.values(balances.tokens) || [];
    const tokenAddresses = tokens.map((t) => t.address).sort();

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

    const TRANSFER_SIGNATURE = api.util.sha3('Transfer(address,address,uint256)');
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

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = _addresses || addressesToFetch;
    const tokens = _tokens || Object.values(balances.tokens);

    if (addresses.length === 0) {
      return Promise.resolve();
    }

    return Promise
      .all(addresses.map((addr) => fetchTokensBalance(addr, tokens, api)))
      .then((tokensBalances) => {
        const balances = {};

        addresses.forEach((addr, idx) => {
          balances[addr] = tokensBalances[idx];
        });

        dispatch(setBalances(balances, skipNotifications));
      })
      .catch((error) => {
        console.warn('balances::fetchTokensBalances', error);
      });
  };
}

function fetchAccount (address, api, full = false) {
  const promises = [ api.eth.getBalance(address) ];

  if (full) {
    promises.push(api.eth.getTransactionCount(address));
  }

  return Promise
    .all(promises)
    .then(([ ethBalance, txCount ]) => {
      const tokens = [ { token: ETH, value: ethBalance } ];
      const balance = { tokens };

      if (full) {
        balance.txCount = txCount;
      }

      return balance;
    })
    .catch((error) => {
      console.warn('balances::fetchAccountBalance', `couldn't fetch balance for account #${address}`, error);
    });
}

function fetchTokensBalance (address, _tokens, api) {
  const tokensPromises = _tokens
    .map((token) => {
      return token.contract.instance.balanceOf.call({}, [ address ]);
    });

  return Promise
    .all(tokensPromises)
    .then((tokensBalance) => {
      const tokens = _tokens
        .map((token, index) => ({
          token,
          value: tokensBalance[index]
        }));

      const balance = { tokens };

      return balance;
    })
    .catch((error) => {
      console.warn('balances::fetchTokensBalance', `couldn't fetch tokens balance for account #${address}`, error);
    });
}

function fetchTokenInfo (tokenreg, tokenId, api, dispatch) {
  return Promise
    .all([
      tokenreg.instance.token.call({}, [tokenId]),
      tokenreg.instance.meta.call({}, [tokenId, 'IMG'])
    ])
    .then(([ tokenData, image ]) => {
      const [ address, tag, format, name ] = tokenData;
      const contract = api.newContract(ABIS.eip20, address);

      const token = {
        format: format.toString(),
        id: tokenId,
        image: hashToImageUrl(image),
        address,
        tag,
        name,
        contract
      };

      return token;
    })
    .catch((error) => {
      console.warn('balances::fetchTokenInfo', `couldn't fetch token #${tokenId}`, error);
    });
}
