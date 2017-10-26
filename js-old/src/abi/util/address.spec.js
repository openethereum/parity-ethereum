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

import { isChecksumValid, isAddress, toChecksumAddress } from './address';

describe('abi/util/address', () => {
  const value = '63Cf90D3f0410092FC0fca41846f596223979195';
  const address = `0x${value}`;
  const lowercase = `0x${value.toLowerCase()}`;
  const uppercase = `0x${value.toUpperCase()}`;
  const invalid = '0x' + value.split('').map((char) => {
    if (char >= 'a' && char <= 'f') {
      return char.toUpperCase();
    } else if (char >= 'A' && char <= 'F') {
      return char.toLowerCase();
    }

    return char;
  }).join('');
  const invalidhex = '0x01234567890123456789012345678901234567gh';

  describe('isChecksumValid', () => {
    it('returns false when fully lowercase', () => {
      expect(isChecksumValid(lowercase)).to.be.false;
    });

    it('returns false when fully uppercase', () => {
      expect(isChecksumValid(uppercase)).to.be.false;
    });

    it('returns false on a mixed-case address', () => {
      expect(isChecksumValid(invalid)).to.be.false;
    });

    it('returns true on a checksummed address', () => {
      expect(isChecksumValid(address)).to.be.true;
    });
  });

  describe('isAddress', () => {
    it('returns true when fully lowercase', () => {
      expect(isAddress(lowercase)).to.be.true;
    });

    it('returns true when fully uppercase', () => {
      expect(isAddress(uppercase)).to.be.true;
    });

    it('returns true when checksummed', () => {
      expect(isAddress(address)).to.be.true;
    });

    it('returns false when invalid checksum', () => {
      expect(isAddress(invalid)).to.be.false;
    });

    it('returns false on valid length, non-hex', () => {
      expect(isAddress(invalidhex)).to.be.false;
    });
  });

  describe('toChecksumAddress', () => {
    it('returns empty when no address specified', () => {
      expect(toChecksumAddress()).to.equal('');
    });

    it('returns empty on invalid address structure', () => {
      expect(toChecksumAddress('0xnotaddress')).to.equal('');
    });

    it('returns formatted address on checksum input', () => {
      expect(toChecksumAddress(address)).to.equal(address);
    });

    it('returns formatted address on lowercase input', () => {
      expect(toChecksumAddress(lowercase)).to.equal(address);
    });

    it('returns formatted address on uppercase input', () => {
      expect(toChecksumAddress(uppercase)).to.equal(address);
    });

    it('returns formatted address on mixed input', () => {
      expect(toChecksumAddress(invalid)).to.equal(address);
    });
  });
});
