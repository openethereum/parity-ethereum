import { api } from '../parity';

import { getTokenTotalSupply } from '../utils';

const {
  isHex,
  isAddressValid,
  toChecksumAddress
} = api.util;

export const ADDRESS_TYPE = 'ADDRESS_TYPE';
export const TOKEN_ADDRESS_TYPE = 'TOKEN_ADDRESS_TYPE';
export const TLA_TYPE = 'TLA_TYPE';
export const UINT_TYPE = 'UINT_TYPE';
export const STRING_TYPE = 'STRING_TYPE';
export const HEX_TYPE = 'HEX_TYPE';

export const ERRORS = {
  invalidTLA: 'The TLA should be 3 characters long',
  invalidUint: 'Please enter a non-negative integer',
  invalidString: 'Please enter at least a character',
  invalidAccount: 'Please select an account to transact with',
  invalidRecipient: 'Please select an account to send to',
  invalidAddress: 'The address is not in the correct format',
  invalidTokenAddress: 'The address is not a regular token contract address',
  invalidHex: 'Please enter an hexadecimal string (digits and letters from a to z)',
  invalidAmount: 'Please enter a positive amount > 0',
  invalidTotal: 'The amount is greater than the availale balance'
};

const validateAddress = (address) => {
  if (!isAddressValid(address)) {
    return {
      error: ERRORS.invalidAddress,
      valid: false
    };
  }

  return {
    value: toChecksumAddress(address),
    error: null,
    valid: true
  };
};

const validateTokenAddress = (address) => {
  let addressValidation = validateAddress(address);
  if (!addressValidation.valid) return addressValidation;

  return getTokenTotalSupply(address)
    .then(balance => {
      if (balance !== null) return addressValidation;

      return {
        error: ERRORS.invalidTokenAddress,
        valid: false
      };
    });
};

const validateTLA = (tla) => {
  if (tla.toString().length !== 3) {
    return {
      error: ERRORS.invalidTLA,
      valid: false
    };
  }

  return {
    value: tla.toString().toUpperCase(),
    error: null,
    valid: true
  };
};

const validateUint = (uint) => {
  if (!/^\d+$/.test(uint) || parseInt(uint) <= 0) {
    return {
      error: ERRORS.invalidUint,
      valid: false
    };
  }

  return {
    value: parseInt(uint),
    error: null,
    valid: true
  };
};

const validateString = (string) => {
  if (string.toString().length === 0) {
    return {
      error: ERRORS.invalidString,
      valid: false
    };
  }

  return {
    value: string.toString(),
    error: null,
    valid: true
  };
};

const validateHex = (string) => {
  if (!isHex(string.toString())) {
    return {
      error: ERRORS.invalidHex,
      valid: false
    };
  }

  return {
    value: string.toString(),
    error: null,
    valid: true
  };
};

export const validate = (value, type) => {
  if (type === ADDRESS_TYPE) return validateAddress(value);
  if (type === TOKEN_ADDRESS_TYPE) return validateTokenAddress(value);
  if (type === TLA_TYPE) return validateTLA(value);
  if (type === UINT_TYPE) return validateUint(value);
  if (type === STRING_TYPE) return validateString(value);
  if (type === HEX_TYPE) return validateHex(value);

  return { valid: true, error: null };
};
