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

import { createHttpApi } from '../../../../test/e2e/ethapi';
import { isBoolean } from '../../../../test/types';

describe('ethapi.net', () => {
  const ethapi = createHttpApi();

  describe('listening', () => {
    it('returns the listening status', () => {
      return ethapi.net.listening().then((status) => {
        expect(isBoolean(status)).to.be.true;
      });
    });
  });

  describe('peerCount', () => {
    it('returns the peer count', () => {
      return ethapi.net.peerCount().then((count) => {
        expect(count.gte(0)).to.be.true;
      });
    });
  });

  describe('version', () => {
    it('returns the version', () => {
      return ethapi.net.version().then((version) => {
        expect(version).to.be.ok;
      });
    });
  });
});
