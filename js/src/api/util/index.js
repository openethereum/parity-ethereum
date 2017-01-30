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

import { isAddress as isAddressValid, toChecksumAddress } from '../../abi/util/address';
import { abiDecode, decodeCallData, decodeMethodInput, methodToAbi } from './decode';
import { abiEncode, encodeMethodCallAbi } from './encode';
import { bytesToHex, hexToAscii, asciiToHex } from './format';
import { fromWei, toWei } from './wei';
import { sha3 } from './sha3';
import { isArray, isFunction, isHex, isInstanceOf, isString } from './types';
import { createIdentityImg } from './identity';

export default {
  abiDecode,
  abiEncode,
  isAddressValid,
  isArray,
  isFunction,
  isHex,
  isInstanceOf,
  isString,
  bytesToHex,
  hexToAscii,
  asciiToHex,
  createIdentityImg,
  decodeCallData,
  decodeMethodInput,
  encodeMethodCallAbi,
  methodToAbi,
  fromWei,
  toChecksumAddress,
  toWei,
  sha3
};
