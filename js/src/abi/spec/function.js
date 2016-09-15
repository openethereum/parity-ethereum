// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import Decoder from '../decoder/decoder';
import Encoder from '../encoder/encoder';
import Param from './param';
import { methodSignature } from '../util/signature';

export default class Func {
  constructor (abi) {
    this._name = abi.name;
    this._constant = !!abi.constant;
    this._inputs = Param.toParams(abi.inputs || []);
    this._outputs = Param.toParams(abi.outputs || []);
    this._signature = methodSignature(this._name, this.inputParamTypes());
  }

  get constant () {
    return this._constant;
  }

  get name () {
    return this._name;
  }

  get inputs () {
    return this._inputs;
  }

  get outputs () {
    return this._outputs;
  }

  get signature () {
    return this._signature;
  }

  inputParamTypes () {
    return this._inputs.map((input) => input.kind);
  }

  outputParamTypes () {
    return this._outputs.map((output) => output.kind);
  }

  encodeCall (tokens) {
    return `${this._signature}${Encoder.encode(tokens)}`;
  }

  decodeOutput (data) {
    return Decoder.decode(this.outputParamTypes(), data);
  }
}
