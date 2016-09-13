import { Logging } from '../subscriptions';

export default class JsonRpcBase {
  constructor () {
    this._id = 1;
    this._debug = false;
  }

  encode (method, params) {
    const json = JSON.stringify({
      jsonrpc: '2.0',
      method: method,
      params: params,
      id: this._id++
    });

    Logging.send(method, params, json);

    return json;
  }

  get id () {
    return this._id;
  }

  get isDebug () {
    return this._debug;
  }

  setDebug (flag) {
    this._debug = flag;
  }

  error (error) {
    if (this.isDebug) {
      console.error(error);
    }
  }

  log (log) {
    if (this.isDebug) {
      console.log(log);
    }
  }
}
