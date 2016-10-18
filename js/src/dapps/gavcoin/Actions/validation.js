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

import { api } from '../parity';

export const ERRORS = {
  invalidAccount: 'please select an account to transact with',
  invalidRecipient: 'please select an account to send to',
  invalidAddress: 'the address is not in the correct format',
  invalidAmount: 'please enter a positive amount > 0',
  invalidTotal: 'the amount is greater than the availale balance'
};

export function validatePositiveNumber (value) {
  let bn = null;

  try {
    bn = new BigNumber(value);
  } catch (e) {
  }

  if (!bn || !bn.gt(0)) {
    return ERRORS.invalidAmount;
  }

  return null;
}

export function validateAccount (account) {
  if (!account || !account.address) {
    return ERRORS.invalidAccount;
  }

  if (!api.util.isAddressValid(account.address)) {
    return ERRORS.invalidAddress;
  }

  account.address = api.util.toChecksumAddress(account.address);

  return null;
}
