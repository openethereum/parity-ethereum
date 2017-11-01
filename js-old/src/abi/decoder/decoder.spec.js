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

import Decoder from './decoder';
import ParamType from '../spec/paramType';
import Token from '../token';
import { padU32 } from '../util/pad';

describe('abi/decoder/Decoder', () => {
  const stringToBytes = function (str) {
    return str.match(/.{1,2}/g).map((code) => parseInt(code, 16));
  };

  const address1 = '0000000000000000000000001111111111111111111111111111111111111111';
  const address2 = '0000000000000000000000002222222222222222222222222222222222222222';
  const address3 = '0000000000000000000000003333333333333333333333333333333333333333';
  const address4 = '0000000000000000000000004444444444444444444444444444444444444444';
  const bool1 = '0000000000000000000000000000000000000000000000000000000000000001';
  const bytes1 = '1234000000000000000000000000000000000000000000000000000000000000';
  const bytes2 = '1000000000000000000000000000000000000000000000000000000000000000';
  const bytes3 = '10000000000000000000000000000000000000000000000000000000000002';
  const bytes4 = '0010000000000000000000000000000000000000000000000000000000000002';
  const int1 = '0111111111111111111111111111111111111111111111111111111111111111';
  const intn = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff85';
  const string1 = '6761766f66796f726b0000000000000000000000000000000000000000000000';
  const string2 = '4665726ee16e64657a0000000000000000000000000000000000000000000000';
  const tokenAddress1 = new Token('address', `0x${address1.slice(-40)}`);
  const tokenAddress2 = new Token('address', `0x${address2.slice(-40)}`);
  const tokenAddress3 = new Token('address', `0x${address3.slice(-40)}`);
  const tokenAddress4 = new Token('address', `0x${address4.slice(-40)}`);
  const tokenBool1 = new Token('bool', true);
  const tokenFixedBytes1 = new Token('fixedBytes', [0x12, 0x34]);
  const tokenBytes1 = new Token('bytes', [0x12, 0x34]);
  const tokenBytes2 = new Token('bytes', stringToBytes(bytes2).concat(stringToBytes(bytes2)));
  const tokenBytes3 = new Token('bytes', stringToBytes(bytes3));
  const tokenBytes4 = new Token('bytes', stringToBytes(bytes4));
  const tokenInt1 = new Token('int', new BigNumber(int1, 16));
  const tokenIntn = new Token('int', new BigNumber(-123));
  const tokenUint1 = new Token('uint', new BigNumber(int1, 16));
  const tokenUintn = new Token('uint', new BigNumber(intn, 16));
  const tokenString1 = new Token('string', 'gavofyork');
  const tokenString2 = new Token('string', 'Fernández');
  const slices = [ address1, address2, address3, address4 ];

  describe('peek', () => {
    it('returns the slice at the correct position', () => {
      expect(Decoder.peek(slices, 1)).to.equal(slices[1]);
    });

    it('returns empty on invalid slices', () => {
      expect(Decoder.peek(null, 4)).to.equal('0000000000000000000000000000000000000000000000000000000000000000');
    });
  });

  describe('takeBytes', () => {
    it('returns a single slice', () => {
      expect(Decoder.takeBytes(slices, 0, 32).bytes).to.deep.equal(stringToBytes(slices[0]));
    });

    it('returns a single partial slice', () => {
      expect(Decoder.takeBytes(slices, 0, 20).bytes).to.deep.equal(stringToBytes(slices[0].substr(0, 40)));
    });

    it('returns multiple slices', () => {
      expect(Decoder.takeBytes(slices, 0, 64).bytes).to.deep.equal(stringToBytes(`${slices[0]}${slices[1]}`));
    });

    it('returns a single offset slice', () => {
      expect(Decoder.takeBytes(slices, 1, 32).bytes).to.deep.equal(stringToBytes(slices[1]));
    });

    it('returns multiple offset slices', () => {
      expect(Decoder.takeBytes(slices, 1, 64).bytes).to.deep.equal(stringToBytes(`${slices[1]}${slices[2]}`));
    });

    it('returns the requires length from slices', () => {
      expect(
        Decoder.takeBytes(slices, 1, 75).bytes
      ).to.deep.equal(stringToBytes(`${slices[1]}${slices[2]}${slices[3]}`.substr(0, 150)));
    });
  });

  describe('decodeParam', () => {
    it('throws an error on non ParamType param', () => {
      expect(() => Decoder.decodeParam({})).to.throw(/ParamType/);
    });

    it('throws an error on invalid param type', () => {
      const pt = new ParamType('address');

      pt._type = 'noMatch';

      expect(() => Decoder.decodeParam(pt)).to.throw(/noMatch/);
    });

    it('decodes an address', () => {
      expect(
        Decoder.decodeParam(new ParamType('address'), [address1], 0).token
      ).to.deep.equal(tokenAddress1);
    });

    it('decodes a bool', () => {
      expect(
        Decoder.decodeParam(new ParamType('bool'), [bool1], 0).token
      ).to.deep.equal(tokenBool1);
    });

    it('decodes an int', () => {
      expect(
        Decoder.decodeParam(new ParamType('int'), [int1], 0).token
      ).to.deep.equal(tokenInt1);
    });

    it('decodes a negative int', () => {
      expect(
        Decoder.decodeParam(new ParamType('int'), [intn], 0).token
      ).to.deep.equal(tokenIntn);
    });

    it('decodes an uint', () => {
      expect(
        Decoder.decodeParam(new ParamType('uint'), [int1], 0).token
      ).to.deep.equal(tokenUint1);
    });

    it('decodes an uint (negative as int)', () => {
      expect(
        Decoder.decodeParam(new ParamType('uint'), [intn], 0).token
      ).to.deep.equal(tokenUintn);
    });

    it('decodes fixedBytes', () => {
      expect(
        Decoder.decodeParam(new ParamType('fixedBytes', null, 2), [bytes1], 0).token
      ).to.deep.equal(tokenFixedBytes1);
    });

    it('decodes bytes', () => {
      expect(
        Decoder.decodeParam(new ParamType('bytes'), [padU32(0x20), padU32(2), bytes1], 0).token
      ).to.deep.equal(tokenBytes1);
    });

    it('decodes string', () => {
      expect(
        Decoder.decodeParam(new ParamType('string'), [padU32(0x20), padU32(9), string1], 0).token
      ).to.deep.equal(tokenString1);
    });

    it('decodes utf8-invalid string', () => {
      expect(
        Decoder.decodeParam(new ParamType('string'), [padU32(0x20), padU32(9), string2], 0).token
      ).to.deep.equal(tokenString2);
    });

    it('decodes string (indexed)', () => {
      expect(
        Decoder.decodeParam(new ParamType('string', null, 0, true), [bytes1], 0)
      ).to.deep.equal(Decoder.decodeParam(new ParamType('fixedBytes', null, 32, true), [bytes1], 0));
    });
  });

  describe('decode', () => {
    it('throws an error on invalid params', () => {
      expect(() => Decoder.decode(null, '123')).to.throw(/array/);
    });

    describe('address', () => {
      it('decodes an address', () => {
        expect(
          Decoder.decode(
            [new ParamType('address')],
            `${address1}`
          )
        ).to.deep.equal([tokenAddress1]);
      });

      it('decodes 2 addresses', () => {
        expect(
          Decoder.decode(
            [new ParamType('address'), new ParamType('address')],
            `${address1}${address2}`
          )
        ).to.deep.equal([tokenAddress1, tokenAddress2]);
      });

      it('decodes a fixedArray of addresses', () => {
        expect(
          Decoder.decode(
            [new ParamType('fixedArray', new ParamType('address'), 2)],
            `${address1}${address2}`
          )
        ).to.deep.equal([new Token('fixedArray', [tokenAddress1, tokenAddress2])]);
      });

      it('decodes a dynamic array of addresses', () => {
        expect(
          Decoder.decode(
            [new ParamType('array', new ParamType('address'))],
            `${padU32(0x20)}${padU32(2)}${address1}${address2}`
          )
        ).to.deep.equal([new Token('array', [tokenAddress1, tokenAddress2])]);
      });

      it('decodes a dynamic array of fixed arrays', () => {
        expect(
          Decoder.decode(
            [new ParamType('array', new ParamType('fixedArray', new ParamType('address'), 2))],
            `${padU32(0x20)}${padU32(2)}${address1}${address2}${address3}${address4}`
          )
        ).to.deep.equal([
          new Token('array', [
            new Token('fixedArray', [tokenAddress1, tokenAddress2]),
            new Token('fixedArray', [tokenAddress3, tokenAddress4])
          ])
        ]);
      });
    });

    describe('int', () => {
      it('decodes an int', () => {
        expect(
          Decoder.decode(
            [new ParamType('int')],
            `${int1}`
          )
        ).to.deep.equal([tokenInt1]);
      });
    });

    describe('uint', () => {
      it('decodes an uint', () => {
        expect(
          Decoder.decode(
            [new ParamType('uint')],
            `${int1}`
          )
        ).to.deep.equal([tokenUint1]);
      });
    });

    describe('fixedBytes', () => {
      it('decodes fixedBytes', () => {
        expect(
          Decoder.decode(
            [new ParamType('fixedBytes', null, 2)],
            `${bytes1}`
          )
        ).to.deep.equal([tokenFixedBytes1]);
      });
    });

    describe('bytes', () => {
      it('decodes bytes', () => {
        expect(
          Decoder.decode(
            [new ParamType('bytes')],
            `${padU32(0x20)}${padU32(2)}${bytes1}`
          )
        ).to.deep.equal([tokenBytes1]);
      });

      it('decodes bytes sequence', () => {
        expect(
          Decoder.decode(
            [new ParamType('bytes')],
            `${padU32(0x20)}${padU32(0x40)}${bytes2}${bytes2}`
          )
        ).to.deep.equal([tokenBytes2]);
      });

      it('decodes bytes seuence (2)', () => {
        expect(
          Decoder.decode(
            [new ParamType('bytes'), new ParamType('bytes')],
            `${padU32(0x40)}${padU32(0x80)}${padU32(0x1f)}${bytes3}00${padU32(0x20)}${bytes4}`
          )
        ).to.deep.equal([tokenBytes3, tokenBytes4]);
      });
    });

    describe('bool', () => {
      it('decodes a single bool', () => {
        expect(
          Decoder.decode(
            [new ParamType('bool')],
            bool1
          )
        ).to.deep.equal([tokenBool1]);
      });
    });

    describe('string', () => {
      it('decodes a string', () => {
        expect(
          Decoder.decode(
            [new ParamType('string')],
            `${padU32(0x20)}${padU32(9)}${string1}`
          )
        ).to.deep.equal([tokenString1]);
      });
    });
  });
});
