// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import Constructor from './constructor';
import Event from './event/event';
import Func from './function';
import Token from '../token';

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
    return Interface.encodeTokens(paramTypes, values);
  }

  static encodeTokens (paramTypes, values) {
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
        case 'fallback':
          return new Func(item);

        default:
          throw new Error(`Unknown ABI type ${item.type}`);
      }
    });
  }
}
