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

import { abiEncode, abiUnencode, abiSignature, encodeMethodCallAbi } from './encode';

const ABI = {
  type: 'function',
  name: 'valid',
  inputs: [
    { type: 'uint256' },
    { type: 'bool' }
  ]
};

const RESULT = [
  'f87fa141',
  '0000000000000000000000000000000000000000000000000000000000000123',
  '0000000000000000000000000000000000000000000000000000000000000001'
].join('');
const VARIABLE = [
  '5a6fbce0',
  'c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470',
  '0000000000000000000000000000000000000000000000000000000000000040',
  '000000000000000000000000000000000000000000000000000000000000000f',
  '687474703a2f2f666f6f2e6261722f0000000000000000000000000000000000'
].join('');

describe('api/util/encode', () => {
  describe('encodeMethodCallAbi', () => {
    it('encodes calls with the correct result', () => {
      expect(encodeMethodCallAbi(ABI, [0x123, true])).to.equal(`0x${RESULT}`);
    });
  });

  describe('abiEncode', () => {
    it('encodes calls with the correct result', () => {
      expect(abiEncode('valid', ['uint256', 'bool'], [0x123, true])).to.equal(`0x${RESULT}`);
    });

    it('encodes variable values', () => {
      expect(
        abiEncode(
          'hintUrl', ['bytes32', 'string'], ['0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470', 'http://foo.bar/']
        )
      ).to.equal(`0x${VARIABLE}`);
    });

    it('encodes only the data with null name', () => {
      expect(
        abiEncode(null, ['uint256', 'bool'], [0x123, true])
      ).to.equal(`0x${RESULT.substr(8)}`);
    });
  });

  describe('abiUnencode', () => {
    it('decodes data correctly from abi', () => {
      expect(
        abiUnencode([{
          name: 'test',
          type: 'function',
          inputs: [
            { type: 'uint', name: 'arga' }
          ]
        }], '0x1acb6f7700000000000000000000000000000038')
      ).to.deep.equal(['test', { arga: 56 }, [56]]);
    });
  });

  // Same example as in abi/util/signature.spec.js
  describe('abiSignature', () => {
    it('encodes baz(uint32,bool) correctly', () => {
      expect(
        abiSignature('baz', ['uint32', 'bool'])
      ).to.equal('0xcdcd77c0992ec5bbfc459984220f8c45084cc24d9b6efed1fae540db8de801d2');
    });
  });
});
