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

import Contracts from '~/contracts';
import { LOG_KEYS, getLogger } from '~/config';
import { fetchTokenIds, fetchTokensBasics, fetchTokensInfo } from '~/util/tokens';

import { setAddressImage } from './imagesActions';

const log = getLogger(LOG_KEYS.Balances);

export function setTokens (tokens) {
  return {
    type: 'setTokens',
    tokens
  };
}

export function loadTokens (options = {}) {
  log.debug('loading tokens', Object.keys(options).length ? options : '');

  return (dispatch, getState) => {
    const { tokenReg } = Contracts.get();

    return tokenReg.getInstance()
      .then((tokenRegInstance) => {
        return fetchTokenIds(tokenRegInstance);
      })
      .then((tokenIndexes) => loadTokensBasics(tokenIndexes, options)(dispatch, getState))
      .catch((error) => {
        console.warn('tokens::loadTokens', error);
      });
  };
}

export function loadTokensBasics (tokenIndexes, options) {
  const limit = 64;
  const count = tokenIndexes.length;

  return (dispatch, getState) => {
    const { api } = getState();
    const { tokenReg } = Contracts.get();
    const tokens = {};

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        let promise = Promise.resolve();

        for (let start = 0; start + limit <= count; start += limit) {
          promise = promise
            .then(() => fetchTokensBasics(api, tokenRegContract, start, limit))
            .then((results) => {
              results.forEach((token) => {
                tokens[token.id] = token;
              });
            });
        }

        return promise;
      })
      .then(() => {
        log.debug('fetched tokens basic info', tokens);

        dispatch(setTokens(tokens));
      })
      .catch((error) => {
        console.warn('tokens::fetchTokens', error);
      });
  };
}

export function fetchTokens (_tokenIndexes, options = {}) {
  const tokenIndexes = uniq(_tokenIndexes || []).sort();
  const tokenChunks = chunk(tokenIndexes, 64);

  return (dispatch, getState) => {
    const { api, images } = getState();
    const { tokenReg } = Contracts.get();

    return tokenReg.getContract()
      .then((tokenRegContract) => {
        let promise = Promise.resolve();

        tokenChunks.forEach((tokenChunk) => {
          promise = promise
            .then(() => fetchTokensInfo(api, tokenRegContract, tokenChunk))
            .then((results) => {
              const tokens = results
                .filter((token) => {
                  return token.name && token.address && !/^(0x)?0*$/.test(token.address);
                })
                .reduce((tokens, token) => {
                  const { id, image, address } = token;

                  // dispatch only the changed images
                  if (images[address] !== image) {
                    dispatch(setAddressImage(address, image, true));
                  }

                  tokens[id] = token;
                  return tokens;
                }, {});

              dispatch(setTokens(tokens));
            });
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
