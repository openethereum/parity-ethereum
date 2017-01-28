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

import Decoder from '../decoder/decoder';
import Encoder from '../encoder/encoder';
import Param from './param';
import { methodSignature } from '../util/signature';

export default class Func {
  constructor (abi) {
    this._abi = abi;
    this._constant = !!abi.constant;
    this._payable = abi.payable;
    this._inputs = Param.toParams(abi.inputs || []);
    this._outputs = Param.toParams(abi.outputs || []);

    const { id, name, signature } = methodSignature(abi.name, this.inputParamTypes());

    this._id = id;
    this._name = name;
    this._signature = signature;
  }

  get abi () {
    return this._abi;
  }

  get constant () {
    return this._constant;
  }

  get name () {
    return this._name;
  }

  get id () {
    return this._id;
  }

  get payable () {
    return this._payable;
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

  decodeInput (data) {
    return Decoder.decode(this.inputParamTypes(), data);
  }

  decodeOutput (data) {
    return Decoder.decode(this.outputParamTypes(), data);
  }
}
