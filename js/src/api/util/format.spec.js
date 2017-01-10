// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { bytesToHex, hexToBytes, hexToAscii, bytesToAscii, asciiToHex, isHex } from './format';

describe('api/util/format', () => {
  describe('bytesToHex', () => {
    it('correctly converts an empty array', () => {
      expect(bytesToHex([])).to.equal('0x');
    });

    it('correctly converts a non-empty array', () => {
      expect(bytesToHex([0, 15, 16])).to.equal('0x000f10');
    });
  });

  describe('hexToBytes', () => {
    it('correctly converts an empty string', () => {
      expect(hexToBytes('')).to.deep.equal([]);
      expect(hexToBytes('0x')).to.deep.equal([]);
    });

    it('correctly converts a non-empty string', () => {
      expect(hexToBytes('0x000f10')).to.deep.equal([0, 15, 16]);
    });
  });

  describe('asciiToHex', () => {
    it('correctly converts an empty string', () => {
      expect(asciiToHex('')).to.equal('0x');
    });

    it('correctly converts a non-empty string', () => {
      expect(asciiToHex('abc')).to.equal('0x616263');
    });
  });

  describe('hexToAscii', () => {
    it('correctly converts an empty string', () => {
      expect(hexToAscii('')).to.equal('');
      expect(hexToAscii('0x')).to.equal('');
    });

    it('correctly converts a non-empty string', () => {
      expect(hexToAscii('0x616263')).to.equal('abc');
    });
  });

  describe('bytesToAscii', () => {
    it('correctly converts an empty string', () => {
      expect(bytesToAscii([])).to.equal('');
    });

    it('correctly converts a non-empty string', () => {
      expect(bytesToAscii([97, 98, 99])).to.equal('abc');
    });
  });

  describe('isHex', () => {
    it('should return true for string starting with 0x', () => {
      expect(isHex('0x123')).to.equal(true);
    });

    it('should return false for non-strings or strings without 0x', () => {
      expect(isHex('120x3')).to.equal(false);
      expect(isHex(false)).to.equal(false);
      expect(isHex()).to.equal(false);
      expect(isHex([1, 2, 3])).to.equal(false);
    });
  });
});
