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

export const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';
export const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

export function isContentHash (url) {
  return /^0x[0-9a-f]{64}/.test(url);
}

/**
 * Convert the given URL to a content hash,
 * and checks if it is already registered in GHH
 */
export const urlToHash = (api, instance, url) => {
  if (!url || !url.length) {
    return Promise.resolve(null);
  }

  const hashPromise = isContentHash(url)
    ? Promise.resolve(url)
    : api.parity.hashContent(url);

  return hashPromise
    .catch((error) => {
      const message = error.text || error.message || error.toString();

      throw new Error(`${message} (${url})`);
    })
    .then((contentHash) => {
      console.log('lookupHash', url, contentHash);

      if (contentHash === INVALID_URL_HASH) {
        throw new Error(`"${url}" is not a valid URL`);
      }

      return instance.entries
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

/**
 * Register the given URL to GithubHint
 * registry contract
 */
export const registerGHH = (instance, url, hash, owner) => {
  const values = [ hash, url ];
  const options = {
    from: owner
  };

  return instance
    .hintURL.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return instance.hintURL.postTransaction(options, values);
    });
};

export const registerDapp = (dappId, dappRegInstance) => {
  const values = [ dappId ];
  const options = {};

  return dappRegInstance
    .fee.call({}, [])
    .then((fee) => {
      options.value = fee;

      return dappRegInstance
        .register.estimateGas(options, values)
        .then((gas) => {
          options.gas = gas.mul(1.2).toFixed(0);
          return dappRegInstance.register.postTransaction(options, values);
        });
    });
};

export const deleteDapp = (dapp, dappRegInstance) => {
  const { id, owner } = dapp;

  const fromAddress = dapp.isOwner
    ? owner.address
    : dapp.contractOwner;

  const values = [ id ];
  const options = {
    from: fromAddress
  };

  return dappRegInstance
    .unregister.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);

      return dappRegInstance.unregister.postTransaction(options, values);
    });
};

export const updateDappOwner = (dappId, dappOwner, nextOwnerAddress, dappRegInstance) => {
  const options = {
    from: dappOwner
  };

  const values = [ dappId, nextOwnerAddress ];

  return dappRegInstance.setDappOwner
    .estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2);

      return dappRegInstance.setDappOwner.postTransaction(options, values);
    });
};

export const updateDapp = (dappId, dappOwner, updates, dappRegInstance, ghhRegInstance) => {
  const options = {
    from: dappOwner
  };

  const types = {
    content: 'CONTENT',
    image: 'IMG',
    manifest: 'MANIFEST'
  };

  const promises = Object
    .keys(updates)
    .map((type) => {
      const key = types[type];
      const url = updates[type];
      let promise;

      if (!url) {
        promise = Promise.resolve([ null, '' ]);
      } else {
        promise = urlToHash(api, ghhRegInstance, url)
          .then((ghhResult) => {
            const { hash, registered } = ghhResult;

            if (!registered) {
              return registerGHH(ghhRegInstance, url, hash, dappOwner)
                .then((requestId) => [ { id: requestId, name: `Registering ${url}` }, hash ]);
            }

            return [ null, hash ];
          });
      }

      return promise
        .then(([ ghhRequest, hash ]) => {
          const values = [ dappId, key, hash ];

          return dappRegInstance.setMeta.estimateGas(options, values)
            .then((gas) => {
              options.gas = gas.mul(1.2).toFixed(0);
              return dappRegInstance.setMeta.postTransaction(options, values);
            })
            .then((requestId) => [ ghhRequest, { id: requestId, name: `Updating ${type} of ${dappId}` } ]);
        });
    });

  if (updates.owner) {
    promises.push(updateDappOwner(updates.owner).then((reqId) => ({ id: reqId, name: `Updating owner of ${dappId}` })));
  }

  return promises;
};
