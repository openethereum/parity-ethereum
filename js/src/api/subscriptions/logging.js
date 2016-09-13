let instance = null;

export default class Logging {
  constructor (updateSubscriptions) {
    this._updateSubscriptions = updateSubscriptions;

    instance = this;
  }

  get isStarted () {
    return true;
  }

  start () {
  }

  static send (method, params, json) {
    if (!instance) {
      return;
    }

    return instance._updateSubscriptions('logging', null, {
      method,
      params,
      json
    });
  }
}
