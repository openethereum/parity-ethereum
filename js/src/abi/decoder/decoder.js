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

import utf8 from 'utf8';

import Token from '../token/token';
import BytesTaken from './bytesTaken';
import DecodeResult from './decodeResult';
import ParamType from '../spec/paramType/paramType';
import { sliceData } from '../util/slice';
import { asAddress, asBool, asI32, asU32 } from '../util/sliceAs';
import { isArray, isInstanceOf } from '../util/types';

const NULL = '0000000000000000000000000000000000000000000000000000000000000000';

export default class Decoder {
  static decode (params, data) {
    if (!isArray(params)) {
      throw new Error('Parameters should be array of ParamType');
    }

    const slices = sliceData(data);
    let offset = 0;

    return params.map((param) => {
      const result = Decoder.decodeParam(param, slices, offset);

      offset = result.newOffset;
      return result.token;
    });
  }

  static peek (slices, position) {
    if (!slices || !slices[position]) {
      return NULL;
    }

    return slices[position];
  }

  static takeBytes (slices, position, length) {
    const slicesLength = Math.floor((length + 31) / 32);
    let bytesStr = '';

    for (let idx = 0; idx < slicesLength; idx++) {
      bytesStr = `${bytesStr}${Decoder.peek(slices, position + idx)}`;
    }

    const bytes = (bytesStr.substr(0, length * 2).match(/.{1,2}/g) || []).map((code) => parseInt(code, 16));

    return new BytesTaken(bytes, position + slicesLength);
  }

  static decodeParam (param, slices, offset) {
    if (!isInstanceOf(param, ParamType)) {
      throw new Error('param should be instanceof ParamType');
    }

    const tokens = [];
    let taken;
    let lengthOffset;
    let length;
    let newOffset;

    switch (param.type) {
      case 'address':
        return new DecodeResult(new Token(param.type, asAddress(Decoder.peek(slices, offset))), offset + 1);

      case 'bool':
        return new DecodeResult(new Token(param.type, asBool(Decoder.peek(slices, offset))), offset + 1);

      case 'int':
        return new DecodeResult(new Token(param.type, asI32(Decoder.peek(slices, offset))), offset + 1);

      case 'uint':
        return new DecodeResult(new Token(param.type, asU32(Decoder.peek(slices, offset))), offset + 1);

      case 'fixedBytes':
        taken = Decoder.takeBytes(slices, offset, param.length);

        return new DecodeResult(new Token(param.type, taken.bytes), taken.newOffset);

      case 'bytes':
        lengthOffset = asU32(Decoder.peek(slices, offset)).div(32).toNumber();
        length = asU32(Decoder.peek(slices, lengthOffset)).toNumber();
        taken = Decoder.takeBytes(slices, lengthOffset + 1, length);

        return new DecodeResult(new Token(param.type, taken.bytes), offset + 1);

      case 'string':
        if (param.indexed) {
          taken = Decoder.takeBytes(slices, offset, 32);

          return new DecodeResult(new Token('fixedBytes', taken.bytes), offset + 1);
        }

        lengthOffset = asU32(Decoder.peek(slices, offset)).div(32).toNumber();
        length = asU32(Decoder.peek(slices, lengthOffset)).toNumber();
        taken = Decoder.takeBytes(slices, lengthOffset + 1, length);

        const str = taken.bytes.map((code) => String.fromCharCode(code)).join('');

        return new DecodeResult(new Token(param.type, utf8.decode(str)), offset + 1);

      case 'array':
        lengthOffset = asU32(Decoder.peek(slices, offset)).div(32).toNumber();
        length = asU32(Decoder.peek(slices, lengthOffset)).toNumber();
        newOffset = lengthOffset + 1;

        for (let idx = 0; idx < length; idx++) {
          const result = Decoder.decodeParam(param.subtype, slices, newOffset);

          newOffset = result.newOffset;
          tokens.push(result.token);
        }

        return new DecodeResult(new Token(param.type, tokens), offset + 1);

      case 'fixedArray':
        newOffset = offset;

        for (let idx = 0; idx < param.length; idx++) {
          const result = Decoder.decodeParam(param.subtype, slices, newOffset);

          newOffset = result.newOffset;
          tokens.push(result.token);
        }

        return new DecodeResult(new Token(param.type, tokens), newOffset);

      default:
        throw new Error(`Invalid param type ${param.type} in decodeParam`);
    }
  }
}
