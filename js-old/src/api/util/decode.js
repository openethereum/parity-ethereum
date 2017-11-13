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

import { isHex } from './types';

import Func from '../../abi/spec/function';
import { fromParamType, toParamType } from '../../abi/spec/paramType/format';

export function decodeCallData (data) {
  if (!isHex(data)) {
    throw new Error('Input to decodeCallData should be a hex value');
  }

  if (data.substr(0, 2) === '0x') {
    return decodeCallData(data.slice(2));
  }

  if (data.length < 8) {
    throw new Error('Input to decodeCallData should be method signature + data');
  }

  const signature = data.substr(0, 8);
  const paramdata = data.substr(8);

  return {
    signature: `0x${signature}`,
    paramdata: `0x${paramdata}`
  };
}

export function decodeMethodInput (methodAbi, paramdata) {
  if (!methodAbi) {
    throw new Error('decodeMethodInput should receive valid method-specific ABI');
  }

  if (paramdata && paramdata.length) {
    if (!isHex(paramdata)) {
      throw new Error('Input to decodeMethodInput should be a hex value');
    }

    if (paramdata.substr(0, 2) === '0x') {
      return decodeMethodInput(methodAbi, paramdata.slice(2));
    }
  }

  return new Func(methodAbi).decodeInput(paramdata).map((decoded) => decoded.value);
}

// takes a method in form name(...,types) and returns the inferred abi definition
export function methodToAbi (method) {
  const length = method.length;
  const typesStart = method.indexOf('(');
  const typesEnd = method.indexOf(')');

  if (typesStart === -1) {
    throw new Error(`Missing start ( in call to decodeMethod with ${method}`);
  } else if (typesEnd === -1) {
    throw new Error(`Missing end ) in call to decodeMethod with ${method}`);
  } else if (typesEnd < typesStart) {
    throw new Error(`End ) is before start ( in call to decodeMethod with ${method}`);
  } else if (typesEnd !== length - 1) {
    throw new Error(`Extra characters after end ) in call to decodeMethod with ${method}`);
  }

  const name = method.substr(0, typesStart);
  const types = method.substr(typesStart + 1, length - (typesStart + 1) - 1).split(',');
  const inputs = types.filter((_type) => _type.length).map((_type) => {
    const type = fromParamType(toParamType(_type));

    return { type };
  });

  return { type: 'function', name, inputs };
}

export function abiDecode (inputTypes, data) {
  return decodeMethodInput({
    inputs: inputTypes.map((type) => {
      return { type };
    })
  }, data);
}
