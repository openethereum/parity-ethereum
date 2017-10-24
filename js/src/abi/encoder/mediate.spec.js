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

import Mediate from './mediate';

describe('abi/encoder/Mediate', () => {
  const LONG15 = '1234567890abcdef000000000000000000000000000000000000000000000000';
  const DOUBLE15 = `${LONG15}${LONG15}`;
  const ARRAY = [new Mediate('raw', DOUBLE15), new Mediate('raw', LONG15)];

  describe('validateType', () => {
    it('validates raw', () => {
      expect(Mediate.validateType('raw')).to.be.true;
    });

    it('validates prefixed', () => {
      expect(Mediate.validateType('prefixed')).to.be.true;
    });

    it('validates fixedArray', () => {
      expect(Mediate.validateType('fixedArray')).to.be.true;
    });

    it('validates array', () => {
      expect(Mediate.validateType('array')).to.be.true;
    });

    it('throws an error on invalid types', () => {
      expect(() => Mediate.validateType('noMatch')).to.throw(/noMatch/);
    });
  });

  describe('offsetFor', () => {
    it('thows an error when offset < 0', () => {
      expect(() => Mediate.offsetFor([1], -1)).to.throw(/Invalid position/);
    });

    it('throws an error when offset >= length', () => {
      expect(() => Mediate.offsetFor([1], 1)).to.throw(/Invalid position/);
    });
  });

  describe('constructor', () => {
    it('throws an error on invalid types', () => {
      expect(() => new Mediate('noMatch', '1')).to.throw(/noMatch/);
    });

    it('sets the type of the object', () => {
      expect((new Mediate('raw', '1')).type).to.equal('raw');
    });

    it('sets the value of the object', () => {
      expect((new Mediate('raw', '1')).value).to.equal('1');
    });
  });

  describe('initLength', () => {
    it('returns correct variable byte length for raw', () => {
      expect(new Mediate('raw', DOUBLE15).initLength()).to.equal(64);
    });

    it('returns correct fixed byte length for array', () => {
      expect(new Mediate('array', [1, 2, 3, 4]).initLength()).to.equal(32);
    });

    it('returns correct fixed byte length for prefixed', () => {
      expect(new Mediate('prefixed', 0).initLength()).to.equal(32);
    });

    it('returns correct variable byte length for fixedArray', () => {
      expect(new Mediate('fixedArray', ARRAY).initLength()).to.equal(96);
    });
  });

  describe('closingLength', () => {
    it('returns 0 byte length for raw', () => {
      expect(new Mediate('raw', DOUBLE15).closingLength()).to.equal(0);
    });

    it('returns prefix + size for prefixed', () => {
      expect(new Mediate('prefixed', DOUBLE15).closingLength()).to.equal(64);
    });

    it('returns prefix + size for array', () => {
      expect(new Mediate('array', ARRAY).closingLength()).to.equal(128);
    });

    it('returns total length for fixedArray', () => {
      expect(new Mediate('fixedArray', ARRAY).closingLength()).to.equal(96);
    });
  });
});
