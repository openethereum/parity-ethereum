import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase
import logger from './logger';

export default class Ws {

  constructor (path = window.location.host, reconnectDelay = 5000) {
    this._path = path;
    this._reconnectTimeout = reconnectDelay;
    this._isConnected = false;
    this._callbacks = {};
    this._queue = [];
    this._id = 1;
    this.onOpen = [];
    this.onMsg = [];
    this.onClose = [];
    this.onError = [];
  }

  init = token => {
    this.token = token; // store token for _onClose reconnect attemps
    clearTimeout(this._initTimeout);
    try {
      const hash = token ? this._hash(token) : null;
      this._ws = new global.WebSocket(`ws://${this._path}`, hash);
    } catch (err) {
      logger.warn('[WS] error connecting to ws', err); // throws when port is blocked, not when hash is incorrect
    }

    this._ws.addEventListener('open', this._onOpen);
    this._ws.addEventListener('error', this._onError);
  }

  send (payload, callback) {
    if (!this._isConnected) {
      logger.log('[WS] not connected. incoming msg added to queue');
      this._queue.push({ payload, callback });
      return;
    }
    const { _id } = this;
    payload.id = _id;
    payload = JSON.stringify(payload);
    this._callbacks[_id] = callback;
    this._ws.send(payload);
    this._id++;
  }

  _onOpen = () => {
    logger.log('[WS] connected');
    this._ws.addEventListener('close', this._onClose);
    this._ws.addEventListener('message', this._onMsg);
    this._isConnected = true;
    this._executeQueue();
    this._triggerEvent(this.onOpen);
  }

  _onMsg = msg => {
    this._triggerEvent(this.onMsg, msg);
    try {
      msg = JSON.parse(msg.data);
    } catch (err) {
      return logger.warn('[WS] unknown msg from server: ', msg, err);
    }
    const cb = this._callbacks[msg.id];
    delete this._callbacks[msg.id];

    if (!cb) {
      return;
    }

    cb(msg.error, msg.result);
  }

  _onClose = () => {
    logger.warn('[WS] closed');
    this._executeCbsWithError();
    this._isConnected = false;
    this._triggerEvent(this.onClose);
    this.init(this.token);
  }

  _onError = err => {
    logger.warn('[WS] error', err);
    this._triggerEvent(this.onError, err);
    this._initTimeout = this._initWithTimeout();
  }

  _triggerEvent (evts, arg) {
    evts.forEach(fn => fn(arg));
  }

  _initWithTimeout () {
    return setTimeout(() => this.init(this.token), this._reconnectTimeout);
  }

  _executeQueue () {
    logger.log('[WS] executing queue: ', this._queue);
    this._queue.forEach(call => {
      this.send(call.payload, call.callback);
    });
    this._queue = [];
  }

  _executeCbsWithError () {
    logger.log('[WS] executing callbacks with error: ', this._callbacks);
    for (const msgId in this._callbacks) {
      const cb = this._callbacks[msgId];
      cb('[WS] disconnected, cb cannot be called');
    }
    this._callbacks = {};
  }

  _hash (token) {
    const time = parseInt(new Date().getTime() / 1000, 10);
    return keccak_256(token + ':' + time) + '_' + time;
  }

  _noop () {}

}
