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

import BytesTaken from './bytesTaken';

describe('abi/decoder/BytesTaken', () => {
  describe('constructor', () => {
    it('sets the bytes of the object', () => {
      expect((new BytesTaken(1, 2)).bytes).to.equal(1);
    });

    it('sets the newOffset of the object', () => {
      expect((new BytesTaken(3, 4)).newOffset).to.equal(4);
    });
  });
});
