export default class WebSocketsProvider {

  constructor (ws) {
    this.ws = ws;
  }

  send (payload) {
    throw Error('[Web3 WS] sync methods are not supported, please use async');
  }

  sendAsync (payload, cb) {
    const { id } = payload;
    this.ws.send(payload, this.formatCb(id, cb));
  }

  formatCb (id, cb) {
    return (err, result) => {
      cb(err, {
        jsonrpc: '2.0',
        id, result
      });
    };
  }

}
