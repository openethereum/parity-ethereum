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

import {
  inAddress, inBlockNumber, inData, inFilter, inHex,
  inNumber10, inNumber16, inOptions, inTraceType,
  inDeriveHash, inDeriveIndex
} from './input';
import { isAddress } from '../../../test/types';

describe('api/format/input', () => {
  const address = '0x63cf90d3f0410092fc0fca41846f596223979195';

  describe('inAddress', () => {
    const address = '63cf90d3f0410092fc0fca41846f596223979195';

    it('adds the leading 0x as required', () => {
      expect(inAddress(address)).to.equal(`0x${address}`);
    });

    it('returns verified addresses as-is', () => {
      expect(inAddress(`0x${address}`)).to.equal(`0x${address}`);
    });

    it('returns lowercase equivalents', () => {
      expect(inAddress(address.toUpperCase())).to.equal(`0x${address}`);
    });

    it('returns 0x on null addresses', () => {
      expect(inAddress()).to.equal('0x');
    });
  });

  describe('inBlockNumber()', () => {
    it('returns earliest as-is', () => {
      expect(inBlockNumber('earliest')).to.equal('earliest');
    });

    it('returns latest as-is', () => {
      expect(inBlockNumber('latest')).to.equal('latest');
    });

    it('returns pending as-is', () => {
      expect(inBlockNumber('pending')).to.equal('pending');
    });

    it('formats existing BigNumber into hex', () => {
      expect(inBlockNumber(new BigNumber(0x123456))).to.equal('0x123456');
    });

    it('formats hex strings into hex', () => {
      expect(inBlockNumber('0x123456')).to.equal('0x123456');
    });

    it('formats numbers into hex', () => {
      expect(inBlockNumber(0x123456)).to.equal('0x123456');
    });
  });

  describe('inData', () => {
    it('formats to hex', () => {
      expect(inData('123456')).to.equal('0x123456');
    });

    it('converts a string to a hex representation', () => {
      expect(inData('jaco')).to.equal('0x6a61636f');
    });
  });

  describe('inHex', () => {
    it('leaves leading 0x as-is', () => {
      expect(inHex('0x123456')).to.equal('0x123456');
    });

    it('adds a leading 0x', () => {
      expect(inHex('123456')).to.equal('0x123456');
    });

    it('returns uppercase as lowercase (leading 0x)', () => {
      expect(inHex('0xABCDEF')).to.equal('0xabcdef');
    });

    it('returns uppercase as lowercase (no leading 0x)', () => {
      expect(inHex('ABCDEF')).to.equal('0xabcdef');
    });

    it('handles empty & null', () => {
      expect(inHex()).to.equal('0x');
      expect(inHex('')).to.equal('0x');
    });
  });

  describe('inFilter', () => {
    ['address'].forEach((input) => {
      it(`formats ${input} address as address`, () => {
        const block = {};

        block[input] = address;
        const formatted = inFilter(block)[input];

        expect(isAddress(formatted)).to.be.true;
        expect(formatted).to.equal(address);
      });
    });

    ['fromBlock', 'toBlock'].forEach((input) => {
      it(`formats ${input} number as blockNumber`, () => {
        const block = {};

        block[input] = 0x123;
        const formatted = inFilter(block)[input];

        expect(formatted).to.equal('0x123');
      });
    });

    it('ignores and passes through unknown keys', () => {
      expect(inFilter({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats an filter options object with relevant entries converted', () => {
      expect(
        inFilter({
          address: address,
          fromBlock: 'latest',
          toBlock: 0x101,
          extraData: 'someExtraStuffInHere',
          limit: 0x32
        })
      ).to.deep.equal({
        address: address,
        fromBlock: 'latest',
        toBlock: '0x101',
        extraData: 'someExtraStuffInHere',
        limit: 50
      });
    });
  });

  describe('inNumber10()', () => {
    it('formats existing BigNumber into number', () => {
      expect(inNumber10(new BigNumber(123))).to.equal(123);
    });

    it('formats hex strings into decimal', () => {
      expect(inNumber10('0x0a')).to.equal(10);
    });

    it('formats numbers into number', () => {
      expect(inNumber10(123)).to.equal(123);
    });

    it('formats undefined into 0', () => {
      expect(inNumber10()).to.equal(0);
    });
  });

  describe('inNumber16()', () => {
    it('formats existing BigNumber into hex', () => {
      expect(inNumber16(new BigNumber(0x123456))).to.equal('0x123456');
    });

    it('formats hex strings into hex', () => {
      expect(inNumber16('0x123456')).to.equal('0x123456');
    });

    it('formats numbers into hex', () => {
      expect(inNumber16(0x123456)).to.equal('0x123456');
    });

    it('formats undefined into 0', () => {
      expect(inNumber16()).to.equal('0x0');
    });
  });

  describe('inOptions', () => {
    ['data'].forEach((input) => {
      it(`converts ${input} to hex data`, () => {
        const block = {};

        block[input] = '1234';
        const formatted = inData(block[input]);

        expect(formatted).to.equal('0x1234');
      });
    });

    ['from', 'to'].forEach((input) => {
      it(`formats ${input} address as address`, () => {
        const block = {};

        block[input] = address;
        const formatted = inOptions(block)[input];

        expect(isAddress(formatted)).to.be.true;
        expect(formatted).to.equal(address);
      });
    });

    it('does not encode an empty `to` value', () => {
      const options = { to: '' };
      const formatted = inOptions(options);

      expect(formatted.to).to.equal('');
    });

    ['gas', 'gasPrice', 'value', 'nonce'].forEach((input) => {
      it(`formats ${input} number as hexnumber`, () => {
        const block = {};

        block[input] = 0x123;
        const formatted = inOptions(block)[input];

        expect(formatted).to.equal('0x123');
      });
    });

    it('passes condition as null when specified as such', () => {
      expect(inOptions({ condition: null })).to.deep.equal({ condition: null });
    });

    it('ignores and passes through unknown keys', () => {
      expect(inOptions({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats an options object with relevant entries converted', () => {
      expect(
        inOptions({
          from: address,
          to: address,
          gas: new BigNumber('0x100'),
          gasPrice: 0x101,
          value: 258,
          nonce: '0x104',
          data: '0123456789',
          extraData: 'someExtraStuffInHere'
        })
      ).to.deep.equal({
        from: address,
        to: address,
        gas: '0x100',
        gasPrice: '0x101',
        value: '0x102',
        nonce: '0x104',
        data: '0x0123456789',
        extraData: 'someExtraStuffInHere'
      });
    });
  });

  describe('inTraceType', () => {
    it('returns array of types as is', () => {
      const types = ['vmTrace', 'trace', 'stateDiff'];

      expect(inTraceType(types)).to.deep.equal(types);
    });

    it('formats single string type into array', () => {
      const type = 'vmTrace';

      expect(inTraceType(type)).to.deep.equal([type]);
    });
  });

  describe('inDeriveHash', () => {
    it('returns derive hash', () => {
      expect(inDeriveHash(1)).to.deep.equal({
        hash: '0x1',
        type: 'soft'
      });

      expect(inDeriveHash(null)).to.deep.equal({
        hash: '0x',
        type: 'soft'
      });

      expect(inDeriveHash({
        hash: 5
      })).to.deep.equal({
        hash: '0x5',
        type: 'soft'
      });

      expect(inDeriveHash({
        hash: 5,
        type: 'hard'
      })).to.deep.equal({
        hash: '0x5',
        type: 'hard'
      });
    });
  });

  describe('inDeriveIndex', () => {
    it('returns derive hash', () => {
      expect(inDeriveIndex(null)).to.deep.equal([]);
      expect(inDeriveIndex([])).to.deep.equal([]);

      expect(inDeriveIndex([1])).to.deep.equal([{
        index: 1,
        type: 'soft'
      }]);

      expect(inDeriveIndex({
        index: 1
      })).to.deep.equal([{
        index: 1,
        type: 'soft'
      }]);

      expect(inDeriveIndex([{
        index: 1,
        type: 'hard'
      }, 5])).to.deep.equal([
        {
          index: 1,
          type: 'hard'
        },
        {
          index: 5,
          type: 'soft'
        }
      ]);
    });
  });
});
