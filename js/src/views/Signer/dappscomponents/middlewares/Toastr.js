
import { addToast, removeToast } from '../actions/toastr';

export default class ToastrMiddleware {

  constructor (time = 6000) {
    this._time = time;
    this._timeouts = {};
  }

  toMiddleware () {
    return store => next => action => {
      const { type, payload } = action;
      if (type === 'remove toast' || type === 'freezeToast') {
        this.clearTimeoutFor(payload);
      }

      if (type === 'add toast') {
        this.onAddToast(store, next, action);
        return;
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
    const { id } = store.getState().toastr;
    const { msg, type } = action.meta.toastr;
    next(addToast({ type, msg, id }));
    this.setTimeoutFor(id, next);
  }

  onAddToast (store, next, action) {
    const { id } = store.getState().toastr;
    action.payload.id = id;
    next(action);
    this.setTimeoutFor(id, next);
  }

  setTimeoutFor (id, next) {
    this._timeouts[String(id)] = setTimeout(() => {
      this.clearTimeoutFor(id);
      next(removeToast(id));
    }, this._time);
  }

  shouldToast (action) {
    return !!(action.meta && action.meta.toastr);
  }

  clearTimeoutFor (id) {
    clearTimeout(this._timeouts[String(id)]);
    delete this._timeouts[String(id)];
  }

}
