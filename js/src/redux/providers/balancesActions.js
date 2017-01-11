// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import imagesEthereum from '../../../assets/images/contracts/ethereum-black-64x64.png';

const ETH = {
  name: 'Ethereum',
  tag: 'ETH',
  image: imagesEthereum
};

function setBalances (_balances) {
  return (dispatch, getState) => {
    const state = getState();

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
      const nextTokens = balance.tokens.slice();

      tokens.forEach((t) => {
        const { token, value } = t;
        const { tag } = token;

        const tokenIndex = nextTokens.findIndex((tok) => tok.token.tag === tag);

        if (tokenIndex === -1) {
          nextTokens.push({
            token,
            value
          });
        } else {
          const oldValue = nextTokens[tokenIndex].value;

          // If received a token/eth (old value < new value), notify
          if (oldValue.lt(value) && accounts[address]) {
            const account = accounts[address];
            const txValue = value.minus(oldValue);

            const redirectToAccount = () => {
              const route = `/account/${account.address}`;
              dispatch(push(route));
            };

            notifyTransaction(account, token, txValue, redirectToAccount);
          }

          nextTokens[tokenIndex] = { token, value };
        }
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

export function loadTokens () {
  return (dispatch, getState) => {
    const { tokenreg } = getState().balances;

    return tokenreg.instance.tokenCount
      .call()
      .then((numTokens) => {
        const tokenIds = range(numTokens.toNumber());
        dispatch(fetchTokens(tokenIds));
      })
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  };
}

export function fetchTokens (_tokenIds) {
  const tokenIds = uniq(_tokenIds || []);
  return (dispatch, getState) => {
    const { api, images, balances } = getState();
    const { tokenreg } = balances;

    return Promise
      .all(tokenIds.map((id) => fetchTokenInfo(tokenreg, id, api)))
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

        dispatch(setTokens(tokens));
        dispatch(fetchBalances());
      })
      .catch((error) => {
        console.warn('balances::fetchTokens', error);
      });
  };
}

export function fetchBalances (_addresses) {
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

        dispatch(setBalances(balances));
        updateTokensFilter(addresses)(dispatch, getState);
      })
      .catch((error) => {
        console.warn('balances::fetchBalances', error);
      });
  };
}

export function updateTokensFilter (_addresses, _tokens) {
  return (dispatch, getState) => {
    const { api, balances, personal } = getState();
    const { visibleAccounts, accounts } = personal;
    const { tokensFilter } = balances;

    const addressesToFetch = uniq(visibleAccounts.concat(Object.keys(accounts)));
    const addresses = uniq(_addresses || addressesToFetch || []).sort();

    const tokens = _tokens || Object.values(balances.tokens) || [];
    const tokenAddresses = tokens.map((t) => t.address).sort();

    if (tokensFilter.filterFromId || tokensFilter.filterToId) {
      const sameTokens = isEqual(tokenAddresses, tokensFilter.tokenAddresses);
      const sameAddresses = isEqual(addresses, tokensFilter.addresses);

      if (sameTokens && sameAddresses) {
        return queryTokensFilter(tokensFilter)(dispatch, getState);
      }
    }

    let promise = Promise.resolve();

    if (tokensFilter.filterFromId) {
      promise = promise.then(() => api.eth.uninstallFilter(tokensFilter.filterFromId));
    }

    if (tokensFilter.filterToId) {
      promise = promise.then(() => api.eth.uninstallFilter(tokensFilter.filterToId));
    }

    if (tokenAddresses.length === 0 || addresses.length === 0) {
      return promise;
    }

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

        dispatch(setTokensFilter(nextTokensFilter));
        fetchTokensBalances(addresses, tokens)(dispatch, getState);
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

export function fetchTokensBalances (_addresses = null, _tokens = null) {
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

        dispatch(setBalances(balances));
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
