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

import BigNumber from 'bignumber.js';

import util from '../api/util';

export const ERRORS = {
  invalidAddress: 'address is an invalid network address',
  duplicateAddress: 'the address is already in your address book',
  invalidChecksum: 'address has failed the checksum formatting',
  invalidName: 'name should not be blank and longer than 2',
  invalidAbi: 'abi should be a valid JSON array',
  invalidCode: 'code should be the compiled hex string',
  invalidNumber: 'invalid number format',
  negativeNumber: 'input number should be positive',
  decimalNumber: 'input number should not contain decimals'
};

export function validateAbi (abi, api) {
  let abiError = null;
  let abiParsed = null;

  try {
    abiParsed = JSON.parse(abi);

    if (!api.util.isArray(abiParsed) || !abiParsed.length) {
      abiError = ERRORS.invalidAbi;
      return { abi, abiError, abiParsed };
    }

    // Validate each elements of the Array
    const invalidIndex = abiParsed
      .map((o) => isValidAbiEvent(o, api) || isValidAbiFunction(o, api) || isAbiFallback(o))
      .findIndex((valid) => !valid);

    if (invalidIndex !== -1) {
      const invalid = abiParsed[invalidIndex];
      abiError = `${ERRORS.invalidAbi} (#${invalidIndex}: ${invalid.name || invalid.type})`;
      return { abi, abiError, abiParsed };
    }

    abi = JSON.stringify(abiParsed);
  } catch (error) {
    abiError = ERRORS.invalidAbi;
  }

  return {
    abi,
    abiError,
    abiParsed
  };
}

function isValidAbiFunction (object, api) {
  if (!object) {
    return false;
  }

  return ((object.type === 'function' && object.name) || object.type === 'constructor') &&
    (object.inputs && api.util.isArray(object.inputs));
}

function isAbiFallback (object) {
  if (!object) {
    return false;
  }

  return object.type === 'fallback';
}

function isValidAbiEvent (object, api) {
  if (!object) {
    return false;
  }

  return (object.type === 'event') &&
    (object.name) &&
    (object.inputs && api.util.isArray(object.inputs));
}

export function validateAddress (address) {
  let addressError = null;

  if (!address) {
    addressError = ERRORS.invalidAddress;
  } else if (!util.isAddressValid(address)) {
    addressError = ERRORS.invalidAddress;
  } else {
    address = util.toChecksumAddress(address);
  }

  return {
    address,
    addressError
  };
}

export function validateCode (code, api) {
  let codeError = null;

  if (!code.length) {
    codeError = ERRORS.invalidCode;
  } else if (!api.util.isHex(code)) {
    codeError = ERRORS.invalidCode;
  }

  return {
    code,
    codeError
  };
}

export function validateName (name) {
  const nameError = !name || name.trim().length < 2 ? ERRORS.invalidName : null;

  return {
    name,
    nameError
  };
}

export function validateUint (value) {
  let valueError = null;

  try {
    const bn = new BigNumber(value);
    if (bn.lt(0)) {
      valueError = ERRORS.negativeNumber;
    } else if (bn.toString().indexOf('.') !== -1) {
      valueError = ERRORS.decimalNumber;
    }
  } catch (e) {
    valueError = ERRORS.invalidNumber;
  }

  return {
    value,
    valueError
  };
}
