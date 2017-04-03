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

import { INVALID_URL_HASH, ZERO_ADDRESS, isContentHash } from './utils';

describe('dapps/dappreg/utils', () => {
  describe('isContentHash', () => {
    it('returns true on valid hashes', () => {
      expect(isContentHash(INVALID_URL_HASH)).to.be.true;
    });

    it('returns false on valid hex, invalid hash', () => {
      expect(isContentHash(ZERO_ADDRESS)).to.be.false;
    });

    it('returns false on invalid hex', () => {
      expect(isContentHash('something')).to.be.false;
    });
  });
});
