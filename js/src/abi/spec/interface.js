import Constructor from './constructor';
import Event from './event/event';
import Func from './function';
import Token from '../token/index';

export default class Interface {
  constructor (abi) {
    this._interface = Interface.parseABI(abi);
  }

  get interface () {
    return this._interface;
  }

  get constructors () {
    return this._interface.filter((item) => item instanceof Constructor);
  }

  get events () {
    return this._interface.filter((item) => item instanceof Event);
  }

  get functions () {
    return this._interface.filter((item) => item instanceof Func);
  }

  encodeTokens (paramTypes, values) {
    const createToken = function (paramType, value) {
      if (paramType.subtype) {
        return new Token(paramType.type, value.map((entry) => createToken(paramType.subtype, entry)));
      }

      return new Token(paramType.type, value);
    };

    return paramTypes.map((paramType, idx) => createToken(paramType, values[idx]));
  }

  static parseABI (abi) {
    return abi.map((item) => {
      switch (item.type) {
        case 'constructor':
          return new Constructor(item);

        case 'event':
          return new Event(item);

        case 'function':
          return new Func(item);

        default:
          throw new Error(`Unknown ABI type ${item.type}`);
      }
    });
  }
}
