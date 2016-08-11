import TYPES from '../spec/paramType/types';

export default class Token {
  constructor (type, value) {
    Token.validateType(type);

    this._type = type;
    this._value = value;
  }

  get type () {
    return this._type;
  }

  get value () {
    return this._value;
  }

  static validateType (type) {
    if (TYPES.filter((_type) => type === _type).length) {
      return true;
    }

    throw new Error(`Invalid type ${type} received for Token`);
  }
}
