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

export function isOwned (contract, hash) {
  return getOwner(contract, hash).then((owner) => !!owner);
}

export function reverse (contract, address) {
  return contract.instance
    .reverse
    .call({}, [ address ]);
}

export function getInfo (contract, name) {
  const ownerPromise = getOwner(contract, name);
  const addressPromise = getMetadata(contract, name, 'A');
  const contentPromise = getMetadata(contract, name, 'CONTENT');
  const imagePromise = getMetadata(contract, name, 'IMG');

  return Promise
    .all([
      ownerPromise,
      addressPromise,
      contentPromise,
      imagePromise
    ])
    .then(([ owner, address, content, image ]) => {
      const result = {
        owner,
        address,
        content,
        image,
        name
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

