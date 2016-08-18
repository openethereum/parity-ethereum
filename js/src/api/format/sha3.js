import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

export function sha3 (value) {
  return keccak_256(value);
}
