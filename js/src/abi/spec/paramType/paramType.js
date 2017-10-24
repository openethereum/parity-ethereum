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

import TYPES from './types';

export default class ParamType {
  constructor (type, subtype = null, length = 0, indexed = false) {
    ParamType.validateType(type);

    this._type = type;
    this._subtype = subtype;
    this._length = length;
    this._indexed = indexed;
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

  get indexed () {
    return this._indexed;
  }

  static validateType (type) {
    if (TYPES.filter((_type) => type === _type).length) {
      return true;
    }

    throw new Error(`Invalid type ${type} received for ParamType`);
  }
}
