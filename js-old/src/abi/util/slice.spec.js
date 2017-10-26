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

import { sliceData } from './slice';

describe('abi/util/slice', () => {
  describe('sliceData', () => {
    const slice1 = '131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b';
    const slice2 = '2124768576358735263578356373526387638357635873563586353756358763';

    it('returns an empty array when length === 0', () => {
      expect(sliceData('')).to.deep.equal([]);
    });

    it('returns an array with the slices otherwise', () => {
      const sliced = sliceData(`${slice1}${slice2}`);

      expect(sliced.length).to.equal(2);
      expect(sliced[0]).to.equal(slice1);
      expect(sliced[1]).to.equal(slice2);
    });

    it('removes leading 0x when passed in', () => {
      const sliced = sliceData(`0x${slice1}${slice2}`);

      expect(sliced.length).to.equal(2);
      expect(sliced[0]).to.equal(slice1);
      expect(sliced[1]).to.equal(slice2);
    });
  });
});
