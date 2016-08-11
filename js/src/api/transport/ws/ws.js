import JsonRpcBase from '../jsonRpcBase';

/* global WebSocket */
export default class Ws extends JsonRpcBase {
  constructor (url, protocols) {
    super();

    this._messages = {};

    this._ws = new WebSocket(url, protocols);
    this._ws.onerror = this._onError;
    this._ws.onopen = this._onOpen;
    this._ws.onclose = this._onClose;
    this._ws.onmessage = this._onMessage;
  }

  _onMessage = (event) => {
    const result = JSON.parse(event.data);
    const {resolve, reject} = this._messages[result.id];

    if (result.error) {
      this.error(event.data);

      reject(new Error(`${result.error.code}: ${result.error.message}`));
      delete this._messages[result.id];
      return;
    }

    this.log(event.data);

    resolve(result.result);
    delete this._messages[result.id];
  }

  execute (method, ...params) {
    return new Promise((resolve, reject) => {
      this._messages[this.id] = { resolve: resolve, reject: reject };
      const json = this.encode(method, params);

      this.log(json);

      this._ws.send(json);
    });
  }
}
