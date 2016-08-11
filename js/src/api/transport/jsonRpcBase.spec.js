import sinon from 'sinon';

import JsonRpcBase from './jsonRpcBase';

const base = new JsonRpcBase();

describe('api/transport/JsonRpcBase', () => {
  describe('encode', () => {
    it('encodes the body correctly, incrementing id', () => {
      const id = base.id;
      const bdy = base.encode('someMethod', ['param1', 'param2']);
      const enc = `{"jsonrpc":"2.0","method":"someMethod","params":["param1","param2"],"id":${id}}`;

      expect(bdy).to.equal(enc);
      expect(base.id - id).to.equal(1);
    });
  });

  describe('setDebug', () => {
    it('starts with disabled flag', () => {
      expect(base.isDebug).to.be.false;
    });

    it('true flag switches on', () => {
      base.setDebug(true);
      expect(base.isDebug).to.be.true;
    });

    it('false flag switches off', () => {
      base.setDebug(true);
      expect(base.isDebug).to.be.true;
      base.setDebug(false);
      expect(base.isDebug).to.be.false;
    });

    describe('logging', () => {
      beforeEach(() => {
        sinon.spy(console, 'log');
        sinon.spy(console, 'error');
      });

      afterEach(() => {
        console.log.restore();
        console.error.restore();
      });

      it('does not log errors with flag off', () => {
        base.setDebug(false);
        base.log('error');
        expect(console.log).to.not.be.called;
      });

      it('does not log errors with flag off', () => {
        base.setDebug(false);
        base.error('error');
        expect(console.error).to.not.be.called;
      });

      it('does log errors with flag on', () => {
        base.setDebug(true);
        base.log('error');
        expect(console.log).to.be.called;
      });

      it('does log errors with flag on', () => {
        base.setDebug(true);
        base.error('error');
        expect(console.error).to.be.called;
      });
    });
  });
});
