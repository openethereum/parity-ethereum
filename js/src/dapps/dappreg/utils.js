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

export const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';
export const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

/**
 * Convert the given URL to a content hash,
 * and checks if it is already registered in GHH
 */
export const urlToHash = (api, instance, url) => {
  if (!url || !url.length) {
    return Promise.resolve(null);
  }

  return api.parity
    .hashContent(url)
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
  const options = {
    from: owner
  };

  const values = [ hash, url ];

  return instance
    .hintURL.estimateGas(options, values)
    .then((gas) => {
      const nextGas = gas.mul(1.2);

      options.gas = nextGas.toFixed(0);
      return instance.hintURL.postTransaction(options, values);
    });
};

export const registerDapp = (dappId, dappRegInstance, dappRegFee) => {
  const values = [ dappId ];
  const options = {
    value: dappRegFee
  };

  console.log('registerDapp', dappId);

  return dappRegInstance
    .register.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return dappRegInstance.register.postTransaction(options, values);
    });
};

export const deleteDapp = (dappId, dappOwner, dappRegInstance) => {
  const values = [ dappId ];
  const options = {
    from: dappOwner
  };

  console.log('deleteDapp', dappId, dappOwner);

  dappRegInstance
    .unregister.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);

      return dappRegInstance.unregister.postTransaction(options, values);
    });
};
