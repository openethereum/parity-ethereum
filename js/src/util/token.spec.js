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

import { retrieveToken } from './token';

describe('retrieveToken', () => {
  it('returns nothing when not found in hash, search or localStorage', () => {
    expect(retrieveToken()).not.to.be.ok;
  });

  describe('localStorage', () => {
    beforeEach(() => {
      localStorage.setItem('sysuiToken', 'yQls-7TX1-t0Jb-0001');
    });

    afterEach(() => {
      localStorage.removeItem('sysuiToken');
    });

    it('retrieves the token', () => {
      expect(retrieveToken()).to.equal('yQls-7TX1-t0Jb-0001');
    });
  });

  describe('URL', () => {
    it('retrieves the token from search', () => {
      expect(
        retrieveToken({
          search: 'token=yQls-7TX1-t0Jb-0002'
        })
      ).to.equal('yQls-7TX1-t0Jb-0002');
    });

    it('retrieves the token from hash', () => {
      expect(
        retrieveToken({
          hash: '#/auth?token=yQls-7TX1-t0Jb-0003'
        })
      ).to.equal('yQls-7TX1-t0Jb-0003');
    });
  });
});
