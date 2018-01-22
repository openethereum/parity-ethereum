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
import { sha3 } from '@parity/api/lib/util/sha3';
import imagesEthereum from '~/../assets/images/contracts/ethereum-black-64x64.png';
import {
  tokenAddresses as tokenAddressesBytcode,
  tokensBalances as tokensBalancesBytecode
} from './bytecodes';

export const ETH_TOKEN = {
  address: '',
  format: new BigNumber(10).pow(18),
  id: getTokenId('eth_native_token'),
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

export function fetchTokensBasics (api, tokenReg, start = 0, limit = 100) {
  const tokenAddressesCallData = encode(
    api,
    [ 'address', 'uint', 'uint' ],
    [ tokenReg.address, start, limit ]
  );

  return api.eth
    .call({ data: tokenAddressesBytcode + tokenAddressesCallData })
    .then((result) => {
      return decodeArray(api, 'address[]', result);
    })
    .then((tokenAddresses) => {
      return tokenAddresses.map((tokenAddress, index) => {
        const tokenIndex = start + index;

        return {
          address: /^0x0*$/.test(tokenAddress)
            ? ''
            : tokenAddress,
          id: getTokenId(tokenIndex),
          index: tokenIndex,
          fetched: false
        };
      });
    })
    .then((tokens) => {
      const randomAddress = sha3(`${Date.now()}`).substr(0, 42);

      return fetchTokensBalances(api, tokens, [randomAddress])
        .then((_balances) => {
          const balances = _balances[randomAddress];

          return tokens.map((token) => {
            if (balances[token.id] && balances[token.id].gt(0)) {
              token.address = '';
            }

            return token;
          });
        });
    });
}

export function fetchTokensInfo (api, tokenReg, tokenIndexes) {
  const requests = tokenIndexes.map((tokenIndex) => {
    const tokenCalldata = tokenReg.getCallData(tokenReg.instance.token, {}, [tokenIndex]);

    return { to: tokenReg.address, data: tokenCalldata };
  });

  const calls = requests.map((req) => api.eth.call(req));
  const imagesPromise = fetchTokensImages(api, tokenReg, tokenIndexes);

  return Promise.all(calls)
    .then((results) => {
      return imagesPromise.then((images) => [ results, images ]);
    })
    .then(([ results, images ]) => {
      return results.map((rawTokenData, index) => {
        const tokenIndex = tokenIndexes[index];
        const tokenData = tokenReg.instance.token
          .decodeOutput(rawTokenData)
          .map((t) => t.value);

        const [ address, tag, format, name ] = tokenData;
        const image = images[index];

        const token = {
          address,
          id: getTokenId(tokenIndex),
          index: tokenIndex,

          format: format.toString(),
          image,
          name,
          tag,

          fetched: true
        };

        return token;
      });
    });
}

export function fetchTokensImages (api, tokenReg, tokenIndexes) {
  const requests = tokenIndexes.map((tokenIndex) => {
    const metaCalldata = tokenReg.getCallData(tokenReg.instance.meta, {}, [tokenIndex, 'IMG']);

    return { to: tokenReg.address, data: metaCalldata };
  });

  const calls = requests.map((req) => api.eth.call(req));

  return Promise.all(calls)
    .then((results) => {
      return results.map((rawImage) => {
        const image = tokenReg.instance.meta.decodeOutput(rawImage)[0].value;

        return hashToImageUrl(image);
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
  const accountAddresses = Object.keys(updates);

  // Updates for the ETH balances
  const ethUpdates = accountAddresses
    .filter((accountAddress) => {
      return updates[accountAddress].find((tokenId) => tokenId === ETH_TOKEN.id);
    })
    .reduce((nextUpdates, accountAddress) => {
      nextUpdates[accountAddress] = [ETH_TOKEN.id];
      return nextUpdates;
    }, {});

  // Updates for Tokens balances
  const tokenUpdates = Object.keys(updates)
    .reduce((nextUpdates, accountAddress) => {
      const tokenIds = updates[accountAddress].filter((tokenId) => tokenId !== ETH_TOKEN.id);

      if (tokenIds.length > 0) {
        nextUpdates[accountAddress] = tokenIds;
      }

      return nextUpdates;
    }, {});

  let ethBalances = {};
  let tokensBalances = {};

  const ethPromise = fetchEthBalances(api, Object.keys(ethUpdates))
    .then((_ethBalances) => {
      ethBalances = _ethBalances;
    });

  const tokenPromise = Object.keys(tokenUpdates)
    .reduce((tokenPromise, accountAddress) => {
      const tokenIds = tokenUpdates[accountAddress];
      const updateTokens = tokens
        .filter((t) => tokenIds.includes(t.id));

      return tokenPromise
        .then(() => fetchTokensBalances(api, updateTokens, [ accountAddress ]))
        .then((balances) => {
          tokensBalances[accountAddress] = balances[accountAddress];
        });
    }, Promise.resolve());

  return Promise.all([ ethPromise, tokenPromise ])
    .then(() => {
      const balances = Object.assign({}, tokensBalances);

      Object.keys(ethBalances).forEach((accountAddress) => {
        if (!balances[accountAddress]) {
          balances[accountAddress] = {};
        }

        balances[accountAddress] = Object.assign(
          {},
          balances[accountAddress],
          ethBalances[accountAddress]
        );
      });

      return balances;
    });
}

function fetchEthBalances (api, accountAddresses) {
  const promises = accountAddresses
    .map((accountAddress) => api.eth.getBalance(accountAddress));

  return Promise.all(promises)
    .then((balancesArray) => {
      return balancesArray.reduce((balances, balance, index) => {
        balances[accountAddresses[index]] = {
          [ETH_TOKEN.id]: balance
        };

        return balances;
      }, {});
    });
}

function fetchTokensBalances (api, tokens, accountAddresses) {
  const tokenAddresses = tokens.map((t) => t.address);
  const tokensBalancesCallData = encode(
    api,
    [ 'address[]', 'address[]' ],
    [ accountAddresses, tokenAddresses ]
  );

  return api.eth
    .call({ data: tokensBalancesBytecode + tokensBalancesCallData })
    .then((result) => {
      const rawBalances = decodeArray(api, 'uint[]', result);
      const balances = {};

      accountAddresses.forEach((accountAddress, accountIndex) => {
        const balance = {};
        const preIndex = accountIndex * tokenAddresses.length;

        tokenAddresses.forEach((tokenAddress, tokenIndex) => {
          const index = preIndex + tokenIndex;
          const token = tokens[tokenIndex];

          balance[token.id] = rawBalances[index];
        });

        balances[accountAddress] = balance;
      });

      return balances;
    });
}

function getTokenId (...args) {
  return sha3(args.join('')).slice(0, 10);
}

function encode (api, types, values) {
  return api.util.abiEncode(
    null,
    types,
    values
  ).replace('0x', '');
}

function decodeArray (api, type, data) {
  return api.util
    .abiDecode(
      [type],
      [
        '0x',
        (32).toString(16).padStart(64, 0),
        data.replace('0x', '')
      ].join('')
    )[0]
    .map((t) => t.value);
}
