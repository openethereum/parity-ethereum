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

import { range } from 'lodash';
import BigNumber from 'bignumber.js';

import { hashToImageUrl } from '~/redux/util';
import { sha3 } from '~/api/util/sha3';
import imagesEthereum from '~/../assets/images/contracts/ethereum-black-64x64.png';

const BALANCEOF_SIGNATURE = sha3('balanceOf(address)');
const ADDRESS_PADDING = range(24).map(() => '0').join('');

export const ETH_TOKEN = {
  address: '',
  format: new BigNumber(10).pow(18),
  id: sha3('eth_native_token').slice(0, 10),
  image: imagesEthereum,
  name: 'Ethereum',
  native: true,
  tag: 'ETH'
};

export function fetchTokenIds (tokenregInstance) {
  return tokenregInstance.tokenCount
    .call()
    .then((numTokens) => {
      const tokenIndexes = range(numTokens.toNumber());

      return tokenIndexes;
    });
}

export function fetchTokenInfo (api, tokenregInstace, tokenIndex) {
  return Promise
    .all([
      tokenregInstace.token.call({}, [tokenIndex]),
      tokenregInstace.meta.call({}, [tokenIndex, 'IMG'])
    ])
    .then(([ tokenData, image ]) => {
      const [ address, tag, format, name ] = tokenData;

      const token = {
        format: format.toString(),
        index: tokenIndex,
        image: hashToImageUrl(image),
        id: sha3(address + tokenIndex).slice(0, 10),
        address,
        name,
        tag
      };

      return token;
    });
}

/**
 * `updates` should be in the shape:
 *   {
 *     [ who ]: [ tokenId ]  // Array of tokens to updates
 *   }
 *
 * Returns a Promise resolved witht the balances in the shape:
 *   {
 *     [ who ]: { [ tokenId ]: BigNumber } // The balances of `who`
 *   }
 */
export function fetchAccountsBalances (api, tokens, updates) {
  const addresses = Object.keys(updates);
  const promises = addresses
    .map((who) => {
      const tokensIds = updates[who];
      const tokensToUpdate = tokensIds.map((tokenId) => tokens.find((t) => t.id === tokenId));

      return fetchAccountBalances(api, tokensToUpdate, who);
    });

  return Promise.all(promises)
    .then((results) => {
      return results.reduce((balances, accountBalances, index) => {
        balances[addresses[index]] = accountBalances;
        return balances;
      }, {});
    });
}

/**
 * Returns a Promise resolved with the balances in the shape:
 *   {
 *     [ tokenId ]: BigNumber  // Token balance value
 *   }
 */
export function fetchAccountBalances (api, tokens, who) {
  const calldata = '0x' + BALANCEOF_SIGNATURE.slice(2, 10) + ADDRESS_PADDING + who.slice(2);
  const promises = tokens.map((token) => fetchTokenBalance(api, token, { who, calldata }));

  return Promise.all(promises)
    .then((results) => {
      return results.reduce((balances, value, index) => {
        const token = tokens[index];

        balances[token.id] = value;
        return balances;
      }, {});
    });
}

export function fetchTokenBalance (api, token, { who, calldata }) {
  if (token.native) {
    return api.eth.getBalance(who);
  }

  return api.eth
    .call({ data: calldata, to: token.address })
    .then((result) => {
      const cleanResult = result.replace(/^0x/, '');

      return new BigNumber(`0x${cleanResult || 0}`);
    });
}
