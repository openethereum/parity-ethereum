
import { addToast, removeToast } from '../actions/toastr';

export default class ToastrMiddleware {

  constructor (time = 4000) {
    this._time = time;
    this._timeouts = {};
  }

  toMiddleware () {
    return store => next => action => {
      const { type, payload } = action;
      if (type === 'remove toast' || type === 'freezeToast') {
        this.clearTimeoutFor(payload);
      }

      // pass along action
      next(action);

      if (!this.shouldToast(action)) {
        return;
      }

      // if action should toast, call next again with toast values
      this.toast(store, next, action);
    };
  }

  toast (store, next, action) {
    const { toastNo } = store.getState().toastr;
    const { msg, type } = action.meta.toastr;
    next(addToast({ type, msg, toastNo }));
    this.setTimeoutFor(toastNo, next);
  }

  setTimeoutFor (toastNo, next) {
    this._timeouts[String(toastNo)] = setTimeout(() => {
      this.clearTimeoutFor(toastNo);
      next(removeToast(toastNo));
    }, this._time);
  }

  shouldToast (action) {
    return !!(action.meta && action.meta.toastr);
  }

  clearTimeoutFor (toastNo) {
    clearTimeout(this._timeouts[String(toastNo)]);
    delete this._timeouts[String(toastNo)];
  }

}
