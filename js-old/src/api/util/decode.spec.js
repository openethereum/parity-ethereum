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

import BigNumber from 'bignumber.js';
import { abiDecode, decodeCallData, decodeMethodInput, methodToAbi } from './decode';

describe('api/util/decode', () => {
  const METH = '0x70a08231';
  const ENCO = '0x70a082310000000000000000000000005A5eFF38DA95b0D58b6C616f2699168B480953C9';
  const DATA = '0x0000000000000000000000005A5eFF38DA95b0D58b6C616f2699168B480953C9';

  describe('decodeCallData', () => {
    it('throws on non-hex inputs', () => {
      expect(() => decodeCallData('invalid')).to.throw(/should be a hex value/);
    });

    it('throws when invalid signature length', () => {
      expect(() => decodeCallData(METH.slice(-6))).to.throw(/should be method signature/);
    });

    it('splits valid inputs properly', () => {
      expect(decodeCallData(ENCO)).to.deep.equal({
        signature: METH,
        paramdata: DATA
      });
    });
  });

  describe('decodeMethodInput', () => {
    it('expects a valid ABI', () => {
      expect(() => decodeMethodInput(null, null)).to.throw(/should receive valid method/);
    });

    it('expect valid hex parameter data', () => {
      expect(() => decodeMethodInput({}, 'invalid')).to.throw(/should be a hex value/);
    });

    it('correctly decodes valid inputs', () => {
      expect(
        decodeMethodInput({
          type: 'function',
          inputs: [
            { type: 'uint' }
          ]
        }, DATA)
      ).to.deep.equal(
        [ new BigNumber('0x5a5eff38da95b0d58b6c616f2699168b480953c9') ]
      );
    });
  });

  describe('methodToAbi', () => {
    it('throws when no start ( specified', () => {
      expect(() => methodToAbi('invalid,uint,bool)')).to.throw(/Missing start \(/);
    });

    it('throws when no end ) specified', () => {
      expect(() => methodToAbi('invalid(uint,bool')).to.throw(/Missing end \)/);
    });

    it('throws when end ) is not in the last position', () => {
      expect(() => methodToAbi('invalid(uint,bool)2')).to.throw(/Extra characters after end \)/);
    });

    it('throws when start ( is after end )', () => {
      expect(() => methodToAbi('invalid)uint,bool(')).to.throw(/End \) is before start \(/);
    });

    it('throws when invalid types are present', () => {
      expect(() => methodToAbi('method(invalidType,bool,uint)')).to.throw(/Cannot convert invalidType/);
    });

    it('returns a valid methodabi for a valid method', () => {
      expect(methodToAbi('valid(uint,bool)')).to.deep.equals({
        type: 'function',
        name: 'valid',
        inputs: [
          { type: 'uint256' },
          { type: 'bool' }
        ]
      });
    });
  });

  describe('abiDecode', () => {
    it('correctly decodes valid inputs', () => {
      expect(abiDecode(['uint'], DATA)).to.deep.equal(
        [ new BigNumber('0x5a5eff38da95b0d58b6c616f2699168b480953c9') ]
      );
    });
  });
});
