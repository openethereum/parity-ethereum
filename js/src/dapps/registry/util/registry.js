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

import { postTransaction } from './transactions';

export function checkOwnerReverse (contract, owner) {
  return contract.instance.canReverse
    .call({}, [ owner ])
    .then((canReverse) => {
      if (!canReverse) {
        return null;
      }

      return contract.instance.reverse.call({}, [ owner ]);
    });
}

export function getInfo (contract, hash) {
  const ownerPromise = getOwner(contract, hash);
  const reverseNamePromise = getReverseName(contract, hash);
  const addressPromise = getMetadata(contract, hash, 'A');
  const contentPromise = getMetadata(contract, hash, 'CONTENT');
  const imagePromise = getMetadata(contract, hash, 'IMG');

  return Promise
    .all([
      ownerPromise,
      addressPromise,
      contentPromise,
      imagePromise,
      reverseNamePromise
    ])
    .then(([ owner, address, content, image, reversedName ]) => {
      const result = {
        owner,
        address,
        content,
        image,
        hash,
        reversedName
      };

      return checkOwnerReverse(contract, owner)
        .then((ownerReverseName) => {
          result.ownerReverseName = ownerReverseName;

          return result;
        });
    });
}

export function getMetadata (contract, hash, key) {
  const { api } = contract;

  const isAddress = key === 'A';
  const method = isAddress
    ? contract.instance.getAddress
    : contract.instance.getData;

  return method.call({}, [ hash, key ])
    .then((_result) => {
      const result = isAddress
        ? _result
        : api.util.bytesToHex(_result);

      return result;
    });
}

export function getOwner (contract, hash) {
  const { address, api } = contract;

  const key = hash + '0000000000000000000000000000000000000000000000000000000000000001';
  const position = api.util.sha3(key, { encoding: 'hex' });

  return api
    .eth
    .getStorageAt(address, position)
    .then((result) => {
      if (/^(0x)?0*$/.test(result)) {
        return '';
      }

      return '0x' + result.slice(-40);
    });
}

export function getReverseName (contract, hash) {
  return contract.instance.hasReverse
    .call({}, [ hash ])
    .then((hasReverse) => {
      if (!hasReverse) {
        return null;
      }

      return contract.instance.getReverse
        .call({}, [ hash ])
        .then((address) => {
          return contract.instance.reverse.call({}, [ address ]);
        });
    });
}

export function isOwned (contract, hash) {
  return getOwner(contract, hash).then((owner) => !!owner);
}

export function modifyMetadata (api, registry, githubHint, owner, hash, key, value) {
  const isAddress = key === 'A';
  const method = isAddress
    ? registry.instance.setAddress
    : registry.instance.setData;

  let nextValuePromise;
  const options = { from: owner };

  // The value is already a hash
  if (/^0x[0-9a-f]+$/i.test(value) || isAddress) {
    nextValuePromise = Promise.resolve(value);
  } else {
    nextValuePromise = api.parity.hashContent(value)
      .then((hashedValue) => {
        return githubHint.instance.entries
          .call({}, [ hashedValue ])
          .then(([ accountSlashRepo, commit, owner ]) => {
            // Not an entry, thus register this entry in GHH contract
            if (/^(0x)0*$/.test(owner)) {
              return postTransaction(api, githubHint.instance.hintURL, options, [ hashedValue, value ]);
            }
          })
          .then(() => {
            return hashedValue;
          });
      });
  }

  return nextValuePromise
    .then((nextValue) => {
      const values = [ hash, key, nextValue ];

      return postTransaction(api, method, options, values);
    });
}

export function reverse (contract, address) {
  return contract.instance
    .reverse
    .call({}, [ address ]);
}
