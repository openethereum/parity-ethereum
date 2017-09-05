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

import { chunkArray } from './array';

describe('util/array', () => {
  describe('chunkArray', () => {
    it('splits array into equal chunks', () => {
      expect(chunkArray([1, 2, 3, 4], 2)).to.deep.equal([[1, 2], [3, 4]]);
    });

    it('splits array into equal chunks (non-divisible)', () => {
      expect(chunkArray([1, 2, 3, 4], 3)).to.deep.equal([[1, 2, 3], [4]]);
    });
  });
});
