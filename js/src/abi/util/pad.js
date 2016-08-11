import BigNumber from 'bignumber.js';
import utf8 from 'utf8';

import { isArray } from './types';

const ZERO_64 = '0000000000000000000000000000000000000000000000000000000000000000';

export function padAddress (input) {
  return `${ZERO_64}${input}`.slice(-64);
}

export function padBool (input) {
  return `${ZERO_64}${input ? '1' : '0'}`.slice(-64);
}

export function padU32 (input) {
  let bn = new BigNumber(input);

  if (bn.lessThan(0)) {
    bn = new BigNumber('ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff', 16)
      .plus(bn).plus(1);
  }

  return `${ZERO_64}${bn.toString(16)}`.slice(-64);
}

export function padBytes (input) {
  const length = isArray(input) ? input.length : (`${input}`.length / 2);

  return `${padU32(length)}${padFixedBytes(input)}`;
}

export function padFixedBytes (input) {
  let sinput;

  if (isArray(input)) {
    sinput = input.map((code) => code.toString(16)).join('');
  } else {
    sinput = `${input}`;
  }

  const max = Math.floor((sinput.length + 63) / 64) * 64;

  return `${sinput}${ZERO_64}`.substr(0, max);
}

export function padString (input) {
  const encoded = utf8.encode(input)
    .split('')
    .map((char) => char.charCodeAt(0).toString(16))
    .join('');

  return padBytes(encoded);
}
