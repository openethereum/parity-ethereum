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

import BigNumber from 'bignumber.js';

import apiutil from '~/api/util';

import { NULL_ADDRESS } from './constants';

// TODO: Convert to FormattedMessages as soon as comfortable with the impact, i.e. errors
// not being concatted into strings in components, all supporting a non-string format
export const ERRORS = {
  invalidAddress: 'address is an invalid network address',
  invalidAmount: 'the supplied amount should be a valid positive number',
  invalidAmountDecimals: 'the supplied amount exceeds the allowed decimals',
  duplicateAddress: 'the address is already in your address book',
  invalidChecksum: 'address has failed the checksum formatting',
  invalidName: 'name should not be blank and longer than 2',
  invalidAbi: 'abi should be a valid JSON array',
  invalidCode: 'code should be the compiled hex string',
  invalidNumber: 'invalid number format',
  negativeNumber: 'input number should be positive',
  decimalNumber: 'input number should not contain decimals',
  gasException: 'the transaction will throw an exception with the current values',
  gasBlockLimit: 'the transaction execution will exceed the block gas limit'
};

export function validateAbi (abi) {
  let abiError = null;
  let abiParsed = null;

  try {
    abiParsed = JSON.parse(abi);

    if (!apiutil.isArray(abiParsed)) {
      abiError = ERRORS.invalidAbi;

      return {
        error: abiError,
        abi,
        abiError,
        abiParsed
      };
    }

    // Validate each elements of the Array
    const invalidIndex = abiParsed
      .map((o) => isValidAbiEvent(o) || isValidAbiFunction(o) || isAbiFallback(o))
      .findIndex((valid) => !valid);

    if (invalidIndex !== -1) {
      const invalid = abiParsed[invalidIndex];

      // TODO: Needs seperate error when using FormattedMessage (no concats)
      abiError = `${ERRORS.invalidAbi} (#${invalidIndex}: ${invalid.name || invalid.type})`;

      return {
        error: abiError,
        abi,
        abiError,
        abiParsed
      };
    }

    abi = JSON.stringify(abiParsed);
  } catch (error) {
    abiError = ERRORS.invalidAbi;
  }

  return {
    error: abiError,
    abi,
    abiError,
    abiParsed
  };
}

function isValidAbiFunction (object) {
  if (!object) {
    return false;
  }

  return ((object.type === 'function' && object.name) || object.type === 'constructor') &&
    (object.inputs && apiutil.isArray(object.inputs));
}

function isAbiFallback (object) {
  if (!object) {
    return false;
  }

  return object.type === 'fallback';
}

function isValidAbiEvent (object) {
  if (!object) {
    return false;
  }

  return (object.type === 'event') &&
    (object.name) &&
    (object.inputs && apiutil.isArray(object.inputs));
}

export function validateAddress (address) {
  let addressError = null;

  if (!address) {
    addressError = ERRORS.invalidAddress;
  } else if (!apiutil.isAddressValid(address)) {
    addressError = ERRORS.invalidAddress;
  } else {
    address = apiutil.toChecksumAddress(address);
  }

  return {
    error: addressError,
    address,
    addressError
  };
}

export function validateCode (code) {
  let codeError = null;

  if (!code || !code.length) {
    codeError = ERRORS.invalidCode;
  } else if (!apiutil.isHex(code)) {
    codeError = ERRORS.invalidCode;
  }

  return {
    error: codeError,
    code,
    codeError
  };
}

export function validateName (name) {
  const nameError = !name || name.trim().length < 2
    ? ERRORS.invalidName
    : null;

  return {
    error: nameError,
    name,
    nameError
  };
}

export function validatePositiveNumber (number) {
  let numberError = null;

  try {
    const v = new BigNumber(number);

    if (v.lt(0)) {
      numberError = ERRORS.invalidAmount;
    }
  } catch (e) {
    numberError = ERRORS.invalidAmount;
  }

  return {
    error: numberError,
    number,
    numberError
  };
}

export function validateDecimalsNumber (number, base = 1) {
  let numberError = null;

  try {
    const s = new BigNumber(number).mul(base).toFixed();

    if (s.indexOf('.') !== -1) {
      numberError = ERRORS.invalidAmountDecimals;
    }
  } catch (e) {
    numberError = ERRORS.invalidAmount;
  }

  return {
    error: numberError,
    number,
    numberError
  };
}

export function validateUint (value) {
  let valueError = null;

  try {
    const bn = new BigNumber(value);

    if (bn.lt(0)) {
      valueError = ERRORS.negativeNumber;
    } else if (!bn.isInteger()) {
      valueError = ERRORS.decimalNumber;
    }
  } catch (e) {
    valueError = ERRORS.invalidNumber;
  }

  return {
    error: valueError,
    value,
    valueError
  };
}

export function isNullAddress (address) {
  if (address && address.substr(0, 2) === '0x') {
    return isNullAddress(address.substr(2));
  }

  return address === NULL_ADDRESS;
}
