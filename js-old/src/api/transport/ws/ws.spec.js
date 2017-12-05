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

import { TEST_WS_URL, mockWs } from '../../../../test/mockRpc';
import Ws from './ws';

describe('api/transport/Ws', () => {
  let transport;
  let scope;

  describe('transport emitter', () => {
    const connect = () => {
      const scope = mockWs();
      const transport = new Ws(TEST_WS_URL);

      return { transport, scope };
    };

    it('emits open event', (done) => {
      const { transport, scope } = connect();

      transport.once('open', () => {
        done();
      });

      scope.stop();
    });

    it('emits close event', (done) => {
      const { transport, scope } = connect();

      transport.once('open', () => {
        scope.server.close();
      });

      transport.once('close', () => {
        done();
      });
    });
  });

  describe('transport', () => {
    let result;

    beforeEach(() => {
      scope = mockWs([{ method: 'test_anyCall', reply: 'TestResult' }]);
      transport = new Ws(TEST_WS_URL);

      return transport
        .execute('test_anyCall', 1, 2, 3)
        .then((_result) => {
          result = _result;
        });
    });

    afterEach(() => {
      scope.stop();
    });

    it('makes call', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('sets jsonrpc', () => {
      expect(scope.body.test_anyCall.jsonrpc).to.equal('2.0');
    });

    it('sets the method', () => {
      expect(scope.body.test_anyCall.method).to.equal('test_anyCall');
    });

    it('passes the params', () => {
      expect(scope.body.test_anyCall.params).to.deep.equal([1, 2, 3]);
    });

    it('increments the id', () => {
      expect(scope.body.test_anyCall.id).not.to.equal(0);
    });

    it('passes the actual result back', () => {
      expect(result).to.equal('TestResult');
    });
  });

  describe('errors', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'test_anyCall', reply: { error: { code: 1, message: 'TestError' } } }]);
      transport = new Ws(TEST_WS_URL);
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns RPC errors when encountered', () => {
      return transport
        .execute('test_anyCall')
        .catch((error) => {
          expect(error).to.match(/TestError/);
        });
    });
  });
});
