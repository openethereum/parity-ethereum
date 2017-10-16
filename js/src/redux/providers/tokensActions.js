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

import { chunk, uniq } from 'lodash';
import store from 'store';

import Contracts from '~/contracts';
import { LOG_KEYS, getLogger } from '~/config';
import { fetchTokenIds, fetchTokensBasics, fetchTokensInfo, fetchTokensImages } from '~/util/tokens';

import { setAddressImage } from './imagesActions';

const TOKENS_CACHE_LS_KEY_PREFIX = '_parity::tokens::';
const log = getLogger(LOG_KEYS.Balances);

function _setTokens (tokens) {
  return {
    type: 'setTokens',
    tokens
  };
}

export function setTokens (nextTokens) {
  return (dispatch, getState) => {
    const { nodeStatus, tokens: prevTokens } = getState();
    const { tokenReg } = Contracts.get();
    const tokens = {
      ...prevTokens,
      ...nextTokens
    };

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        const lsKey = TOKENS_CACHE_LS_KEY_PREFIX + nodeStatus.netChain;

        store.set(lsKey, {
          tokenreg: tokenRegContract.address,
          tokens
        });
      })
      .catch((error) => {
        console.error(error);
      })
      .then(() => {
        dispatch(_setTokens(nextTokens));
      });
  };
}

function loadCachedTokens (tokenRegContract) {
  return (dispatch, getState) => {
    const { nodeStatus } = getState();

    const lsKey = TOKENS_CACHE_LS_KEY_PREFIX + nodeStatus.netChain;
    const cached = store.get(lsKey);

    if (cached) {
      // Check if we have data from the right contract
      if (cached.tokenreg === tokenRegContract.address && cached.tokens) {
        log.debug('found cached tokens', cached.tokens);
        dispatch(_setTokens(cached.tokens));

        // Fetch all the tokens images on load
        // (it's the only thing that might have changed)
        const tokenIndexes = Object.values(cached.tokens)
          .filter((t) => t && t.fetched)
          .map((t) => t.index);

        fetchTokensData(tokenRegContract, tokenIndexes)(dispatch, getState);
      } else {
        store.remove(lsKey);
      }
    }
  };
}

export function loadTokens (options = {}) {
  log.debug('loading tokens', Object.keys(options).length ? options : '');

  return (dispatch, getState) => {
    const { tokenReg } = Contracts.get();

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        loadCachedTokens(tokenRegContract)(dispatch, getState);
        return fetchTokenIds(tokenRegContract.instance);
      })
      .then((tokenIndexes) => loadTokensBasics(tokenIndexes, options)(dispatch, getState))
      .catch((error) => {
        console.warn('tokens::loadTokens', error);
      });
  };
}

export function loadTokensBasics (_tokenIndexes, options) {
  const limit = 64;

  return (dispatch, getState) => {
    const { api, tokens } = getState();
    const { tokenReg } = Contracts.get();
    const nextTokens = {};
    const prevTokensIndexes = Object.values(tokens).map((t) => t.index);

    // Only fetch tokens we don't have yet
    const tokenIndexes = _tokenIndexes
      .filter((tokenIndex) => {
        return !prevTokensIndexes.includes(tokenIndex);
      })
      .sort();

    const count = tokenIndexes.length;

    log.debug('loading basic tokens', tokenIndexes);

    if (count === 0) {
      return Promise.resolve();
    }

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        let promise = Promise.resolve();
        const first = tokenIndexes[0];
        const last = tokenIndexes[tokenIndexes.length - 1];

        for (let from = first; from <= last; from += limit) {
          // No need to fetch `limit` elements
          const lowerLimit = Math.min(limit, last - from + 1);

          promise = promise
            .then(() => fetchTokensBasics(api, tokenRegContract, from, lowerLimit))
            .then((results) => {
              results
                .forEach((token) => {
                  nextTokens[token.id] = token;
                });
            });
        }

        return promise;
      })
      .then(() => {
        log.debug('fetched tokens basic info', nextTokens);

        dispatch(setTokens(nextTokens));
      })
      .catch((error) => {
        console.warn('tokens::fetchTokens', error);
      });
  };
}

export function fetchTokens (_tokenIndexes) {
  const tokenIndexes = uniq(_tokenIndexes || []);
  const tokenChunks = chunk(tokenIndexes, 64);

  return (dispatch, getState) => {
    const { tokenReg } = Contracts.get();

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        let promise = Promise.resolve();

        tokenChunks.forEach((tokenChunk) => {
          promise = promise
            .then(() => fetchTokensData(tokenRegContract, tokenChunk)(dispatch, getState));
        });

        return promise;
      })
      .then(() => {
        log.debug('fetched token', getState().tokens);
      })
      .catch((error) => {
        console.warn('tokens::fetchTokens', error);
      });
  };
}

/**
 * Split the given token indexes between those for whom
 * we already have some info, and thus just need to fetch
 * the image, and those for whom we don't have anything and
 * need to fetch all the info.
 */
function fetchTokensData (tokenRegContract, tokenIndexes) {
  return (dispatch, getState) => {
    const { api, tokens, images } = getState();
    const allTokens = Object.values(tokens);

    const tokensIndexesMap = allTokens
      .reduce((map, token) => {
        map[token.index] = token;
        return map;
      }, {});

    const fetchedTokenIndexes = allTokens
      .filter((token) => token.fetched)
      .map((token) => token.index);

    const fullIndexes = [];
    const partialIndexes = [];

    tokenIndexes.forEach((tokenIndex) => {
      if (fetchedTokenIndexes.includes(tokenIndex)) {
        partialIndexes.push(tokenIndex);
      } else {
        fullIndexes.push(tokenIndex);
      }
    });

    log.debug('need to fully fetch', fullIndexes);
    log.debug('need to partially fetch', partialIndexes);

    const fullPromise = fetchTokensInfo(api, tokenRegContract, fullIndexes);
    const partialPromise = fetchTokensImages(api, tokenRegContract, partialIndexes)
      .then((imagesResult) => {
        return imagesResult.map((image, index) => {
          const tokenIndex = partialIndexes[index];
          const token = tokensIndexesMap[tokenIndex];

          return { ...token, image };
        });
      });

    return Promise.all([ fullPromise, partialPromise ])
      .then(([ fullResults, partialResults ]) => {
        log.debug('fetched', { fullResults, partialResults });

        return [].concat(fullResults, partialResults)
          .reduce((tokens, token) => {
            const { id, image, address } = token;

            // dispatch only the changed images
            if (images[address] !== image) {
              dispatch(setAddressImage(address, image, true));
            }

            tokens[id] = token;
            return tokens;
          }, {});
      })
      .then((tokens) => {
        dispatch(setTokens(tokens));
      });
  };
}
