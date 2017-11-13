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

import { bytesToHex, cleanupValue, hexToBytes, hexToAscii, bytesToAscii, asciiToHex } from './format';

describe('api/util/format', () => {
  describe('bytesToHex', () => {
    it('correctly converts an empty array', () => {
      expect(bytesToHex([])).to.equal('0x');
    });

    it('correctly converts a non-empty array', () => {
      expect(bytesToHex([0, 15, 16])).to.equal('0x000f10');
    });
  });

  describe('cleanupValue', () => {
    it('returns unknown values as the original', () => {
      expect(cleanupValue('original', 'unknown')).to.equal('original');
    });

    it('returns ascii arrays as ascii', () => {
      expect(cleanupValue([97, 115, 99, 105, 105, 0], 'bytes32')).to.equal('ascii');
    });

    it('returns non-ascii arrays as hex strings', () => {
      expect(cleanupValue([97, 200, 0, 0], 'bytes4')).to.equal('0x61c80000');
    });

    it('returns uint (>48) as the original', () => {
      expect(cleanupValue('original', 'uint49')).to.equal('original');
    });

    it('returns uint (<=48) as the number value', () => {
      expect(cleanupValue('12345', 'uint48')).to.equal(12345);
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
      expect(asciiToHex('a\nb')).to.equal('0x610a62');
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
});
