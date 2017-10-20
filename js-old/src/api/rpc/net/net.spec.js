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

import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';
import { isBigNumber } from '../../../../test/types';

import Http from '../../transport/http';
import Net from './net';

const instance = new Net(new Http(TEST_HTTP_URL, -1));

describe('api/rpc/Net', () => {
  describe('peerCount', () => {
    it('returns the connected peers, formatted', () => {
      mockHttp([{ method: 'net_peerCount', reply: { result: '0x123456' } }]);

      return instance.peerCount().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(0x123456)).to.be.true;
      });
    });
  });
});
