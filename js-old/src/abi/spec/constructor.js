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

import Encoder from '../encoder/encoder';
import Param from './param';

export default class Constructor {
  constructor (abi) {
    this._inputs = Param.toParams(abi.inputs || []);
  }

  get inputs () {
    return this._inputs;
  }

  inputParamTypes () {
    return this._inputs.map((input) => input.kind);
  }

  encodeCall (tokens) {
    return Encoder.encode(tokens);
  }
}
