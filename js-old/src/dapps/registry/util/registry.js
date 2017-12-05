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

export const getOwner = (contract, name) => {
  const { address, api } = contract;

  const key = api.util.sha3.text(name) + '0000000000000000000000000000000000000000000000000000000000000001';
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
};

export const isOwned = (contract, name) => {
  return getOwner(contract, name).then((owner) => !!owner);
};
