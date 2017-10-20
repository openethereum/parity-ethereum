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

import Http from '../../transport/http';
import Trace from './trace';

const instance = new Trace(new Http(TEST_HTTP_URL, -1));

describe('api/rpc/Trace', () => {
  let scope;

  describe('block', () => {
    beforeEach(() => {
      scope = mockHttp([{ method: 'trace_block', reply: { result: [] } }]);
    });

    it('assumes latest blockNumber when not specified', () => {
      return instance.block().then(() => {
        expect(scope.body.trace_block.params).to.deep.equal(['latest']);
      });
    });

    it('passed specified blockNumber', () => {
      return instance.block(0x123).then(() => {
        expect(scope.body.trace_block.params).to.deep.equal(['0x123']);
      });
    });
  });
});
