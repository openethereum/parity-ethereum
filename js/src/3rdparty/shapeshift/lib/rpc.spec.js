const helpers = require('../helpers.spec.js');
const { APIKEY, mockget, mockpost, rpc } = helpers;

describe('lib/rpc', () => {
  describe('GET', () => {
    const REPLY = { test: 'this is some result' };

    let scope;
    let result;

    beforeEach(() => {
      scope = mockget([{ path: 'test', reply: REPLY }]);

      return rpc
        .get('test')
        .then((_result) => {
          result = _result;
        });
    });

    it('does GET', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('retrieves the info', () => {
      expect(result).to.deep.equal(REPLY);
    });
  });

  describe('POST', () => {
    const REPLY = { test: 'this is some result' };

    let scope;
    let result;

    beforeEach(() => {
      scope = mockpost([{ path: 'test', reply: REPLY }]);

      return rpc
        .post('test', { input: 'stuff' })
        .then((_result) => {
          result = _result;
        });
    });

    it('does POST', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('retrieves the info', () => {
      expect(result).to.deep.equal(REPLY);
    });

    it('passes the input object', () => {
      expect(scope.body.test.input).to.equal('stuff');
    });

    it('passes the apikey specified', () => {
      expect(scope.body.test.apiKey).to.equal(APIKEY);
    });
  });
});
