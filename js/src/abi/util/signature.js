import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase
import { fromParamType } from '../spec/paramType/format';

export function eventSignature (name, params) {
  const types = (params || []).map(fromParamType).join(',');
  const id = `${name || ''}(${types})`;

  return keccak_256(id);
}

export function methodSignature (name, params) {
  return eventSignature(name, params).substr(0, 8);
}
