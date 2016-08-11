import { padAddress, padBool, padBytes, padFixedBytes, padU32, padString } from '../util/pad';
import Mediate from './mediate';
import Token from '../token/token';
import { isArray, isInstanceOf } from '../util/types';

export default class Encoder {
  static encode (tokens) {
    if (!isArray(tokens)) {
      throw new Error('tokens should be array of Token');
    }

    const mediates = tokens.map((token) => Encoder.encodeToken(token));
    const inits = mediates
      .map((mediate, idx) => mediate.init(Mediate.offsetFor(mediates, idx)))
      .join('');
    const closings = mediates
      .map((mediate, idx) => mediate.closing(Mediate.offsetFor(mediates, idx)))
      .join('');

    return `${inits}${closings}`;
  }

  static encodeToken (token) {
    if (!isInstanceOf(token, Token)) {
      throw new Error('token should be instanceof Token');
    }

    switch (token.type) {
      case 'address':
        return new Mediate('raw', padAddress(token.value));

      case 'int':
      case 'uint':
        return new Mediate('raw', padU32(token.value));

      case 'bool':
        return new Mediate('raw', padBool(token.value));

      case 'fixedBytes':
        return new Mediate('raw', padFixedBytes(token.value));

      case 'bytes':
        return new Mediate('prefixed', padBytes(token.value));

      case 'string':
        return new Mediate('prefixed', padString(token.value));

      case 'fixedArray':
      case 'array':
        return new Mediate(token.type, token.value.map((token) => Encoder.encodeToken(token)));

      default:
        throw new Error(`Invalid token type ${token.type} in encodeToken`);
    }
  }
}
