// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { sha3 } from './sha3';

describe('api/util/sha3', () => {
  describe('sha3', () => {
    it('constructs a correct sha3 value', () => {
      expect(sha3('jacogr')).to.equal('0x2f4ff4b5a87abbd2edfed699db48a97744e028c7f7ce36444d40d29d792aa4dc');
    });
  });
});
