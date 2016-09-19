// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import format from '../api/format';

export const ERRORS = {
  invalidAddress: 'address is an invalid network address',
  duplicateAddress: 'the address is already in your address book',
  invalidChecksum: 'address has failed the checksum formatting',
  invalidName: 'name should not be blank and longer than 2'
};

export function validateAddress (address) {
  let addressError = null;

  if (!address) {
    addressError = ERRORS.invalidAddress;
  } else if (!format.isAddressValid(address)) {
    addressError = ERRORS.invalidAddress;
  } else {
    address = format.toChecksumAddress(address);
  }

  return {
    address,
    addressError
  };
}

export function validateName (name) {
  const nameError = !name || name.length < 2 ? ERRORS.invalidName : null;

  return {
    name,
    nameError
  };
}
