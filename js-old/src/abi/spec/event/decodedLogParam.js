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

import ParamType from '../paramType/paramType';
import Token from '../../token/token';
import { isInstanceOf } from '../../util/types';

export default class DecodedLogParam {
  constructor (name, kind, token) {
    if (!isInstanceOf(kind, ParamType)) {
      throw new Error('kind not instanceof ParamType');
    } else if (!isInstanceOf(token, Token)) {
      throw new Error('token not instanceof Token');
    }

    this._name = name;
    this._kind = kind;
    this._token = token;
  }

  get name () {
    return this._name;
  }

  get kind () {
    return this._kind;
  }

  get token () {
    return this._token;
  }
}
