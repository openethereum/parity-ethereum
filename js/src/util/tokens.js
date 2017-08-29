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

import { flatten, range } from 'lodash';
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

export function fetchTokensInfo (api, tokenReg, tokenIndexes) {
  const requests = tokenIndexes.map((tokenIndex) => {
    const tokenCalldata = tokenReg.getCallData(tokenReg.instance.token, {}, [tokenIndex]);
    const metaCalldata = tokenReg.getCallData(tokenReg.instance.meta, {}, [tokenIndex, 'IMG']);

    return [
      { to: tokenReg.address, data: tokenCalldata },
      { to: tokenReg.address, data: metaCalldata }
    ];
  });

  return api.parity.call(flatten(requests))
    .then((results) => {
      return tokenIndexes.map((tokenIndex, index) => {
        const [ rawTokenData, rawImage ] = results.slice(index * 2, index * 2 + 2);

        const tokenData = tokenReg.instance.token
          .decodeOutput(rawTokenData)
          .map((t) => t.value);

        const image = tokenReg.instance.meta.decodeOutput(rawImage)[0].value;

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
    });
}

/**
 * `updates` should be in the shape:
 *   {
 *     [ who ]: [ tokenId ]  // Array of tokens to updates
 *   }
 *
 * Returns a Promise resolved with the balances in the shape:
 *   {
 *     [ who ]: { [ tokenId ]: BigNumber } // The balances of `who`
 *   }
 */
export function fetchAccountsBalances (api, tokens, updates) {
  const tokenIdMap = tokens.reduce((map, token) => {
    map[token.id] = token;
    return map;
  }, {});

  const addresses = Object.keys(updates);

  const ethUpdates = addresses
    .map((who) => {
      const tokensIds = updates[who];

      if (tokensIds.includes(ETH_TOKEN.id)) {
        return who;
      }
    })
    .filter((who) => who);

  // An Array which each elements is an Object
  // containing the request Object (with data and to fields),
  // and the who field
  const calls = addresses
    .map((who) => {
      const tokensIds = updates[who];

      return tokensIds
        .map((id) => tokenIdMap[id])
        // Filter out non-contract tokens
        .filter((t) => t.address)
        .map((token) => {
          const calldata = '0x' + BALANCEOF_SIGNATURE.slice(2, 10) + ADDRESS_PADDING + who.slice(2);

          return {
            request: {
              to: token.address,
              data: calldata
            },
            who,
            tokenId: token.id
          };
        });
    })
    .reduce((calls, requests) => [].concat(calls, requests), []);

  const requests = calls.map((c) => c.request);

  return Promise
    .all([
      api.parity.call(requests),
      Promise.all(ethUpdates.map((who) => api.eth.getBalance(who)))
    ])
    .then(([ _tokensResults, _ethResults ]) => {
      const balances = {};

      const tokensResults = _tokensResults.map((result, index) => {
        const { who, tokenId } = calls[index];
        const cleanValue = result.replace(/^0x/, '');

        return { who, tokenId, value: new BigNumber(`0x${cleanValue || 0}`) };
      });

      const ethResults = _ethResults.map((balance, index) => {
        return {
          value: balance,
          who: ethUpdates[index],
          tokenId: ETH_TOKEN.id
        };
      });

      [].concat(ethResults, tokensResults).forEach((result) => {
        const { value, who, tokenId } = result;

        if (!balances[who]) {
          balances[who] = {};
        }

        balances[who][tokenId] = value;
      });

      return balances;
    });
}
