import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';
import Http from './http';

const transport = new Http(TEST_HTTP_URL);

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
