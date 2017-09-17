// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { uniq } from 'lodash';
import store from 'store';

import Api from './api';
import { LOG_KEYS, getLogger } from '~/config';

const log = getLogger(LOG_KEYS.Signer);

export default class SecureApi extends Api {
  _isConnecting = false;
  _needsToken = false;
  _tokens = [];
  _uiApi = null;

  _dappsUrl = null;
  _wsUrl = null;
  _url = null;

  static getTransport (url, sysuiToken, protocol) {
    const transportUrl = SecureApi.transportUrl(url, protocol);

    return new Api.Transport.Ws(transportUrl, sysuiToken, false);
  }

  static transportUrl (url, protocol) {
    const proto = protocol() === 'https:' ? 'wss:' : 'ws:';

    return `${proto}//${url}`;
  }

  // Returns a protocol with `:` at the end.
  static protocol () {
    return window.location.protocol;
  }

  constructor (uiUrl, nextToken, getTransport = SecureApi.getTransport, protocol = SecureApi.protocol) {
    const sysuiToken = store.get('sysuiToken');
    const transport = getTransport(uiUrl, sysuiToken, protocol);

    super(transport);

    this.protocol = protocol;
    this._url = uiUrl;
    this._uiApi = new Api(new Api.Transport.Http(`${this.protocol()}//${this._url}/rpc`, 0), false);
    this._wsUrl = uiUrl;
    // Try tokens from localStorage, from hash and 'initial'
    this._tokens = uniq([sysuiToken, nextToken, 'initial'])
      .filter((token) => token)
      .map((token) => ({ value: token, tried: false }));

    // When the transport is closed, try to reconnect
    transport.on('close', this.connect, this);

    this.connect();
  }

  get _dappsAddress () {
    if (!this._dappsUrl) {
      return {
        host: null,
        port: 8545
      };
    }

    const [host, port] = this._dappsUrl.split(':');

    return {
      host,
      port: parseInt(port, 10)
    };
  }

  get dappsPort () {
    return this._dappsAddress.port;
  }

  get dappsUrl () {
    const { port } = this._dappsAddress;

    return `${this.protocol()}//${this.hostname}:${port}`;
  }

  get hostname () {
    if (window.location.hostname === 'home.parity') {
      return 'dapps.parity';
    }

    return this._dappsAddress.host;
  }

  get isConnecting () {
    return this._isConnecting;
  }

  get isConnected () {
    return this._transport.isConnected;
  }

  get needsToken () {
    return this._needsToken;
  }

  get secureToken () {
    return this._transport.token;
  }

  connect () {
    if (this._isConnecting) {
      return;
    }

    log.debug('trying to connect...');

    this._isConnecting = true;

    this.emit('connecting');

    // Reset the tested Tokens
    this._resetTokens();

    // Try to connect
    return this._connect()
      .then((connected) => {
        this._isConnecting = false;

        if (connected) {
          const token = this.secureToken;

          log.debug('got connected ; saving token', token);

          // Save the sucessful token
          this._saveToken(token);
          this._needsToken = false;

          // Emit the connected event
          return this.emit('connected');
        }

        // If not connected, we need a new token
        log.debug('needs a token');
        this._needsToken = true;

        return this.emit('disconnected');
      })
      .catch((error) => {
        this._isConnecting = false;

        log.debug('emitting "disconnected"');
        this.emit('disconnected');
        console.error('unhandled error in secureApi', error);
      });
  }

  /**
   * Resolves a wildcard address to `window.location.hostname`;
   */
  _resolveHost (url) {
    const parts = url ? url.split(':') : [];
    const port = parts[1];
    let host = parts[0];

    if (!host) {
      return host;
    }

    if (host === '0.0.0.0') {
      host = window.location.hostname;
    }

    return port ? `${host}:${port}` : host;
  }

  /**
   * Returns a Promise that gets resolved with
   * a boolean: `true` if the node is up, `false`
   * otherwise (HEAD request to the Node)
   */
  isNodeUp () {
    return fetch(`${this.protocol()}//${this._url}/api/ping`, { method: 'HEAD' })
      .then(
        (r) => r.status === 200,
        () => false
      )
      .catch(() => false);
  }

  /**
   * Update the given token, ie. add it to the token
   * list, and then try to connect (if not already connecting)
   */
  updateToken (_token) {
    const token = this._sanitiseToken(_token);

    log.debug('updating token', token);

    // Update the tokens list: put the new one on first position
    this._tokens = [ { value: token, tried: false } ].concat(this._tokens);

    // Try to connect with the new token added
    return this.connect();
  }

  /**
   * Try to connect to the Node with the next Token in
   * the list
   */
  _connect () {
    log.debug('trying next token');

    // Get the first not-tried token
    const nextToken = this._getNextToken();

    // If no more tokens to try, user has to enter a new one
    if (!nextToken) {
      return Promise.resolve(false);
    }

    nextToken.tried = true;

    return this._connectWithToken(nextToken.value)
      .then((validToken) => {
        // If not valid, try again with the next token in the list
        if (!validToken) {
          return this._connect();
        }

        // If correct and valid token, wait until the Node is ready
        // and resolve as connected
        return this._waitUntilNodeReady()
          .then(() => true);
      })
      .catch((error) => {
        log.error('unkown error in _connect', error);
        return false;
      });
  }

  /**
   * Connect with the given token.
   * It returns a Promise that gets resolved
   * with `validToken` as argument, whether the given token
   * is valid or not
   */
  _connectWithToken (_token) {
    // Sanitize the token first
    const token = this._sanitiseToken(_token);

    const connectPromise = this._fetchSettings()
      .then(() => {
        // Update the URL and token in the transport layer
        this.transport.url = SecureApi.transportUrl(this._wsUrl, this.protocol);
        this.transport.updateToken(token, false);

        log.debug('connecting with token', token);

        return this.transport.connect();
      })
      .then(() => {
        log.debug('connected with', token);

        if (token === 'initial') {
          return this._generateAuthorizationToken();
        }

        // The token is valid !
        return true;
      })
      .catch((error) => {
        // Log if it's not a close error (ie. wrong token)
        if (error && error.type !== 'close') {
          log.debug('did not connect ; error', error);
        }

        return false;
      });

    return Promise
      .all([
        connectPromise,
        this.isNodeUp()
      ])
      .then(([ connected, isNodeUp ]) => {
        if (connected) {
          return true;
        }

        // If it's not up, try again in a few...
        if (!isNodeUp) {
          const timeout = this.transport.retryTimeout;

          log.debug('node is not up ; will try again in', timeout, 'ms');

          return new Promise((resolve, reject) => {
            window.setTimeout(() => {
              this._connectWithToken(token).then(resolve).catch(reject);
            }, timeout);
          });
        }

        // The token is invalid
        log.debug('tried with a wrong token', token);
        return false;
      });
  }

  /**
   * Retrieve the correct ports from the Node
   */
  _fetchSettings () {
    return Promise
      .all([
        // ignore dapps disabled errors
        this._uiApi.parity.dappsUrl().catch(() => null),
        this._uiApi.parity.wsUrl()
      ])
      .then(([dappsUrl, wsUrl]) => {
        this._dappsUrl = this._resolveHost(dappsUrl);
        this._wsUrl = this._resolveHost(wsUrl);
      });
  }

  /**
   * Try to generate an Authorization Token.
   * Then try to connect with the new token.
   */
  _generateAuthorizationToken () {
    return this.signer
      .generateAuthorizationToken()
      .then((token) => this._connectWithToken(token));
  }

  /**
   * Get the next token to try, if any left
   */
  _getNextToken () {
    // Get the first not-tried token
    const nextTokenIndex = this._tokens.findIndex((t) => !t.tried);

    // If no more tokens to try, user has to enter a new one
    if (nextTokenIndex < 0) {
      return null;
    }

    const nextToken = this._tokens[nextTokenIndex];

    return nextToken;
  }

  _resetTokens () {
    this._tokens = this._tokens.map((token) => ({
      ...token,
      tried: false
    }));
  }

  _sanitiseToken (token) {
    return token.replace(/[^a-zA-Z0-9]/g, '');
  }

  _saveToken (token) {
    store.set('sysuiToken', token);
  }

  /**
   * Promise gets resolved when the node is up
   * and running (it might take some time before
   * the node is actually ready even when the client
   * is connected).
   *
   * We check that the `parity_netChain` RPC calls
   * returns successfully
   */
  _waitUntilNodeReady (_timeleft) {
    // Default timeout to 30 seconds
    const timeleft = Number.isFinite(_timeleft)
      ? _timeleft
      : 30 * 1000;

    // After timeout, just resolve the promise...
    if (timeleft <= 0) {
      console.warn('node is still not ready after 30 seconds...');
      return Promise.resolve(true);
    }

    const start = Date.now();

    return this
      .parity.netChain()
      .then(() => true)
      .catch((error) => {
        if (!error) {
          return true;
        }

        if (error.type !== 'NETWORK_DISABLED') {
          throw error;
        }

        // Timeout between 250ms and 750ms
        const timeout = Math.floor(250 + (500 * Math.random()));

        log.debug('waiting until node is ready', 'retry in', timeout, 'ms');

        // Retry in a few...
        return new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const duration = Date.now() - start;

            this._waitUntilNodeReady(timeleft - duration).then(resolve).catch(reject);
          }, timeout);
        });
      });
  }
}
