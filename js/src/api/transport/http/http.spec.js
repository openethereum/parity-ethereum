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
import Http from './http';

const transport = new Http(TEST_HTTP_URL, -1);

describe('api/transport/Http', () => {
  describe('instance', () => {
    it('encodes the options correctly', () => {
      const opt = transport._encodeOptions('someMethod', ['param']);
      const enc = {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
          'Content-Length': 65
        },
        body: `{"jsonrpc":"2.0","method":"someMethod","params":["param"],"id":${transport._id - 1}}`
      };

      expect(opt).to.deep.equal(enc);
    });
  });

  describe('transport emitter', () => {
    it('emits close event', (done) => {
      transport.once('close', () => {
        done();
      });

      transport.execute('eth_call');
    });

    it('emits open event', (done) => {
      mockHttp([{ method: 'eth_call', reply: { result: '' } }]);

      transport.once('open', () => {
        done();
      });

      transport.execute('eth_call');
    });
  });

  describe('transport', () => {
    const RESULT = ['this is some result'];

    let scope;
    let result;

    beforeEach(() => {
      scope = mockHttp([{ method: 'eth_call', reply: { result: RESULT } }]);

      return transport
        .execute('eth_call', 1, 2, 3, 'test')
        .then((_result) => {
          result = _result;
        });
    });

    it('makes POST', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('sets jsonrpc', () => {
      expect(scope.body.eth_call.jsonrpc).to.equal('2.0');
    });

    it('sets the method', () => {
      expect(scope.body.eth_call.method).to.equal('eth_call');
    });

    it('passes the params', () => {
      expect(scope.body.eth_call.params).to.deep.equal([1, 2, 3, 'test']);
    });

    it('increments the id', () => {
      expect(scope.body.eth_call.id).not.to.equal(0);
    });

    it('passes the actual result back', () => {
      expect(result).to.deep.equal(RESULT);
    });
  });

  describe('HTTP errors', () => {
    let scope;
    let error;

    beforeEach(() => {
      scope = mockHttp([{ method: 'eth_call', reply: {}, code: 500 }]);

      return transport
        .execute('eth_call')
        .catch((_error) => {
          error = _error;
        });
    });

    it('returns HTTP errors as throws', () => {
      expect(scope.isDone()).to.be.true;
      expect(error.message).to.match(/Internal Server Error/);
    });
  });

  describe('RPC errors', () => {
    const ERROR = { code: -1, message: 'ERROR: RPC failure' };

    let scope;
    let error;

    beforeEach(() => {
      scope = mockHttp([{ method: 'eth_call', reply: { error: ERROR } }]);

      return transport
        .execute('eth_call')
        .catch((_error) => {
          error = _error;
        });
    });

    it('returns RPC errors as throws', () => {
      expect(scope.isDone()).to.be.true;
      expect(error.message).to.match(/RPC failure/);
    });
  });
});
