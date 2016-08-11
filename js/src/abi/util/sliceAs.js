import BigNumber from 'bignumber.js';

export function asU32 (slice) {
  // TODO: validation

  return new BigNumber(slice, 16);
}

export function asI32 (slice) {
  if (new BigNumber(slice.substr(0, 1), 16).toString(2)[0] === '1') {
    return new BigNumber(slice, 16)
      .minus(new BigNumber('ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff', 16))
      .minus(1);
  }

  return new BigNumber(slice, 16);
}

export function asAddress (slice) {
  // TODO: address validation?

  return slice.slice(-40);
}

export function asBool (slice) {
  // TODO: everything else should be 0

  return new BigNumber(slice[63]).eq(1);
}
