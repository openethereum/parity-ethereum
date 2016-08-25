import BigNumber from 'bignumber.js';

export const ERRORS = {
  invalidAccount: 'please select an account to transact from',
  invalidAmount: 'please enter a positive amount > 0'
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
  return account && account.address
    ? null
    : ERRORS.invalidAccount;
}
