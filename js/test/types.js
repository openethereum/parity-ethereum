import BigNumber from 'bignumber.js';
import { isInstanceOf } from '../src/api/util/types';

export { isFunction, isInstanceOf } from '../src/api/util/types';
export { isAddress } from '../src/api/format/address';

const ZEROS = '000000000000000000000000000000000000000000000000000000000000';

export function isBigNumber (test) {
  return isInstanceOf(test, BigNumber);
}

export function isBoolean (test) {
  return Object.prototype.toString.call(test) === '[object Boolean]';
}

export function isHexNumber (_test) {
  if (_test.substr(0, 2) !== '0x') {
    return false;
  }

  const test = _test.substr(2);
  const hex = `${ZEROS}${(new BigNumber(_test, 16)).toString(16)}`.slice(-1 * test.length);

  return hex === test;
}
