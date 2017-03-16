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

import { api } from './parity';

import { eip20 as eip20Abi } from '~/contracts/abi';

export const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';
export const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

/**
 * Convert the given URL to a content hash,
 * and checks if it is already registered in GHH
 */
export const urlToHash = (ghhInstance, url) => {
  if (!url || !url.length) {
    return Promise.resolve(null);
  }

  return api.parity
    .hashContent(url)
    .catch((error) => {
      const message = error.text || error.message || error.toString();

      throw new Error(`${message} (${url})`);
    })
    .then((contentHash) => {
      console.log('lookupHash', url, contentHash);

      if (contentHash === INVALID_URL_HASH) {
        throw new Error(`"${url}" is not a valid URL`);
      }

      return ghhInstance.entries
        .call({}, [contentHash])
        .then(([accountSlashRepo, commit, contentHashOwner]) => {
          const registered = (contentHashOwner !== ZERO_ADDRESS);

          return {
            hash: contentHash,
            registered
          };
        });
    });
};

export const getTokenTotalSupply = (tokenAddress) => {
  return api
    .eth
    .getCode(tokenAddress)
    .then(code => {
      if (!code || /^(0x)?0?$/.test(code)) {
        return null;
      }

      const contract = api.newContract(eip20Abi, tokenAddress);

      return contract
        .instance
        .totalSupply
        .call({}, []);
    });
};
