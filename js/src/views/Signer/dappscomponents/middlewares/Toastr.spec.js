import sinon from 'sinon';
import ToastrMiddleware from './Toastr';
import { removeToast, addToast } from '../actions/toastr';

describe('MIDDLEWARE: TOASTR', () => {
  let cut, state;
  const time = 20;
  let toastNo = 1;

  beforeEach('mock cut', () => {
    cut = new ToastrMiddleware(time);
    state = {
      toastr: {
        id: toastNo
      }
    };
  });

  describe('TO MIDDLEWARE', () => {
    beforeEach('mock methods', () => {
      cut.clearTimeoutFor = sinon.spy();
      cut.toast = sinon.spy();
    });

    it('should call only clearTimeoutFor and next, when respected action is dispatched', () => {
      // given
      const store = null;
      const next = sinon.spy();
      const middleware = cut.toMiddleware()(store)(next);
      const action = removeToast(toastNo);
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(cut.clearTimeoutFor.calledWith(toastNo)).to.be.true;
      expect(next.calledWith(action)).to.be.true;
      expect(cut.toast.called).to.be.false;
    });

    it('should call only next when non-respected action is dispatched', () => {
      // given
      const store = null;
      const next = sinon.spy();
      const middleware = cut.toMiddleware()(store)(next);
      const action = { type: 'test' };
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(cut.clearTimeoutFor.called).to.be.false;
      expect(next.calledWith(action)).to.be.true;
      expect(cut.toast.called).to.be.false;
    });

    it('should call only next and toast, when action with meta toastr is dispatched', () => {
      // given
      const msg = 'test';
      const store = null;
      const next = sinon.spy();
      const middleware = cut.toMiddleware()(store)(next);
      const meta = { toastr: { msg, type: 'default' } };
      const action = { type: 'test', payload: 'test', meta };
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(cut.clearTimeoutFor.called).to.be.false;
      expect(next.calledWith(action)).to.be.true;
      expect(cut.toast.calledWith(store, next, action)).to.be.true;
    });
  });

  describe('TOAST', () => {
    // using 'before' doesn't work
    // it might b overwriten by the global beforeEach
    beforeEach('spy on removeToast', () => {
      cut.setTimeoutFor = sinon.spy();
    });

    it('should call next and setTimeoutFor', () => {
      // given
      const msg = 'text';
      const type = 'default';
      const store = { getState: () => state };
      const next = sinon.spy();
      const action = { meta: { toastr: {
        msg, type
      } } };

      // when
      cut.toast(store, next, action);

      // then
      expect(next.calledWith(addToast({
        msg, type, id: toastNo
      }))).to.be.true;

      expect(cut.setTimeoutFor.calledWith(toastNo, next));
    });
  });

  describe('SET TIMEOUT FOR', () => {
    beforeEach('spy on clearTimeoutFor', () => {
      cut.clearTimeoutFor = sinon.spy();
    });
    it('should call clearTimeoutFor and next after cut._time', done => {
      // given
      const next = sinon.spy();

      // when
      cut.setTimeoutFor(toastNo, next);

      // then
      expect(cut._timeouts[String(toastNo)]).to.be.an('object');
      setTimeout(() => {
        expect(cut.clearTimeoutFor.calledWith(toastNo)).to.be.true;
        expect(next.calledWith(removeToast(toastNo))).to.be.true;
        done();
      }, time);
    });
  });

  describe('SHOULD TOAST', () => {
    it('should return false when action isn\'t toastable', () => {
      // given
      const action = { meta: {} };

      // when
      const res = cut.shouldToast(action);

      // then
      expect(res).to.be.false;
    });
    it('should return true when action is toastable', () => {
      // given
      const action = { meta: { toastr: { msg: 'foo' } } };

      // when
      const res = cut.shouldToast(action);

      // then
      expect(res).to.be.true;
    });
  });
  describe('CLEAR TIMEOUT FOR', () => {
    let mockedTimeoutSpy;
    beforeEach('mock timeouts', () => {
      mockedTimeoutSpy = sinon.spy();
      cut._timeouts[String(toastNo)] = setTimeout(() => {
        mockedTimeoutSpy();
      }, time);
    });
    it('should clear and delete timeout', done => {
      // when
      cut.clearTimeoutFor(toastNo);

      // then
      expect(cut._timeouts[String(toastNo)]).to.be.undefined;
      setTimeout(() => {
        expect(mockedTimeoutSpy.called).to.be.false;
        done();
      }, time);
    });
  });
});
