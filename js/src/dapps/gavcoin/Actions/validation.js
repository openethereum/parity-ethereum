import BigNumber from 'bignumber.js';

const { Api } = window.parity;

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

  // 0x7b4983739f097f1d4fa18460826ec6828014b633

  if (!Api.format.isAddressValid(account.address)) {
    return ERRORS.invalidAddress;
  }

  account.address = Api.format.toChecksumAddress(account.address);

  return null;
}
