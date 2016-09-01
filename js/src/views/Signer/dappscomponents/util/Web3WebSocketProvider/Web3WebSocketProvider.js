/* global WebSocket */
export default class Web3WebSocketProvider {

  constructor (host = 'localhost', port = 8180, path) {
    this.path = path || `ws://${host}:${port}`;
    this.ws = new WebSocket(this.path);
    this.ws.addEventListener('open', this.onOpen);
    this.ws.addEventListener('message', this.onMessage);
    this.callbacks = {};
    this.queue = []; // hold calls until ws is connected on init or if disconnected
    this.id = 0;
  }

  onOpen = evt => {
    console.log('WS: listening on: ', this.path);
    this.isWsConnected = true;
    this.executeQueue();
  };

  onMessage = msg => {
    console.log('WS: incoming msg: ', msg);
    try {
      msg = JSON.parse(msg.data);
    } catch (err) {
      return console.error('error parsing msg from WS: ', msg, err);
    }
    const cb = this.callbacks[msg.id];
    delete this.callbacks[msg.id];
    if (!cb) {
      return;
    }
    cb(null, msg); // web3 uses error first cb style
  }

  send (payload) {
    throw new Error('404: websockets dont support sync calls');
  }

  sendAsync (payload, cb) {
    console.log('WS: send async: ', payload, 'with cb: ', !!cb);
    if (!this.isWsConnected) {
      this.queue.push({ payload, cb });
      return console.log('WS: incoming msg when not connected, adding to queue');
    }
    this.id++;
    const { id } = this;
    this.ws.send(JSON.stringify(payload));
    if (!cb) {
      return;
    }

    this.callbacks[id] = cb;
  }

  executeQueue () {
    console.log('WS: executing queue: ', this.queue);
    this.queue.forEach(call => {
      this.sendAsync(call.payload, call.cb);
    });
  }

  // Compatibility with rest of W3 providers
  isConnected () {
    return this.isWsConnected;
  }

}
