import sinon from 'sinon';
import logger from './logger';

describe('MIDDLEWARE: LOGGER', () => {
  describe('MIDDLEWARE', () => {
    const state = { logger: { logging: true } };

    beforeEach('spy console', () => {
      sinon.spy(console, 'log');
      sinon.spy(console, 'error');
    });

    afterEach('unspy console', () => {
      console.log.restore();
      console.error.restore();
    });

    it('should call console.log on non-error msgs', () => {
      // given
      const store = { getState: () => state };
      const next = sinon.spy();
      const action = { type: 'test action' };
      const middleware = logger(store)(next);
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(console.error.called).to.be.false;
      expect(console.log.calledOnce).to.be.true;
    });

    it('should call console.log on non-error msgs', () => {
      // given
      const store = { getState: () => state };
      const next = sinon.spy();
      const action = { type: 'test error action' };
      const middleware = logger(store)(next);
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(console.log.called).to.be.false;
      expect(console.error.calledOnce).to.be.true;
    });
  });
});
