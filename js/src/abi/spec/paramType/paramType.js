import TYPES from './types';

export default class ParamType {
  constructor (type, subtype, length) {
    ParamType.validateType(type);

    this._type = type;
    this._subtype = subtype;
    this._length = length;
  }

  get type () {
    return this._type;
  }

  get subtype () {
    return this._subtype;
  }

  get length () {
    return this._length;
  }

  static validateType (type) {
    if (TYPES.filter((_type) => type === _type).length) {
      return true;
    }

    throw new Error(`Invalid type ${type} received for ParamType`);
  }
}
