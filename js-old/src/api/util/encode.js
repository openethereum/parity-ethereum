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

import Abi from '~/abi';
import Func from '~/abi/spec/function';

import { abiDecode } from './decode';
import { cleanupValue } from './format';
import { sha3 } from './sha3';

export function encodeMethodCallAbi (methodAbi = {}, values = []) {
  const func = new Func(methodAbi);
  const tokens = Abi.encodeTokens(func.inputParamTypes(), values);
  const call = func.encodeCall(tokens);

  return `0x${call}`;
}

export function abiEncode (methodName, inputTypes, data) {
  const result = encodeMethodCallAbi({
    name: methodName || '',
    type: 'function',
    inputs: inputTypes.map((type) => {
      return { type };
    })
  }, data);

  return result;
}

export function abiUnencode (abi, data) {
  const callsig = data.substr(2, 8);
  const op = abi.find((field) => {
    return field.type === 'function' &&
      abiSignature(field.name, field.inputs.map((input) => input.type)).substr(2, 8) === callsig;
  });

  if (!op) {
    console.warn(`Unknown function ID: ${callsig}`);
    return null;
  }

  let argsByIndex = abiDecode(op.inputs.map((field) => field.type), '0x' + data.substr(10))
    .map((value, index) => cleanupValue(value, op.inputs[index].type));
  const argsByName = op.inputs.reduce((result, field, index) => {
    result[field.name] = argsByIndex[index];

    return result;
  }, {});

  return [op.name, argsByName, argsByIndex];
}

export function abiSignature (name, inputs) {
  return sha3(`${name}(${inputs.join()})`);
}
