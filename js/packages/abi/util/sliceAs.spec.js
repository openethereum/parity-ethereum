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

import { asAddress, asBool, asI32, asU32 } from './sliceAs';

describe('abi/util/sliceAs', () => {
  const MAX_INT = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';

  describe('asAddress', () => {
    it('correctly returns the last 0x40 characters', () => {
      const address = '1111111111222222222233333333334444444444';

      expect(asAddress(`000000000000000000000000${address}`)).to.equal(`0x${address}`);
    });
  });

  describe('asBool', () => {
    it('correctly returns true', () => {
      expect(asBool('0000000000000000000000000000000000000000000000000000000000000001')).to.be.true;
    });

    it('correctly returns false', () => {
      expect(asBool('0000000000000000000000000000000000000000000000000000000000000000')).to.be.false;
    });
  });

  describe('asI32', () => {
    it('correctly decodes positive numbers', () => {
      expect(asI32('000000000000000000000000000000000000000000000000000000000000007b').toString()).to.equal('123');
    });

    it('correctly decodes negative numbers', () => {
      expect(asI32('ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff85').toString()).to.equal('-123');
    });
  });

  describe('asU32', () => {
    it('returns a maxium U32', () => {
      expect(asU32(MAX_INT).toString(16)).to.equal(MAX_INT);
    });
  });
});
