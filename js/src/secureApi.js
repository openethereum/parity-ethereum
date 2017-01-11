// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Api from './api';
import { LOG_KEYS, getLogger } from '~/config';

const log = getLogger(LOG_KEYS.Signer);
const sysuiToken = window.localStorage.getItem('sysuiToken');

export default class SecureApi extends Api {

  _isConnecting = false;
  _needsToken = false;
  _tokens = [];

  _dappsInterface = null;
  _dappsPort = 8080;
  _signerPort = 8180;

  constructor (url, nextToken) {
    const transport = new Api.Transport.Ws(url, sysuiToken, false);
    super(transport);

    this._url = url;

    // Try tokens from localstorage, from hash and 'initial'
    this._tokens = uniq([sysuiToken, nextToken, 'initial'])
      .filter((token) => token)
      .map((token) => ({ value: token, tried: false }));

    // When the transport is closed, try to reconnect
    transport.on('close', this.connect, this);
    this.connect();
  }

  get dappsPort () {
    return this._dappsPort;
  }

  get dappsUrl () {
    return `http://${this.hostname}:${this.dappsPort}`;
  }

  get hostname () {
    if (window.location.hostname === 'home.parity') {
      return 'dapps.parity';
    }

    if (!this._dappsInterface || this._dappsInterface === '0.0.0.0') {
      return window.location.hostname;
    }

    return this._dappsInterface;
  }

  get signerPort () {
    return this._signerPort;
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

    log.debug('emitting "connecting"');
    this.emit('connecting');

    // Reset the tested Tokens
    this._resetTokens();

    // Try the next Token, ie. the first one
    return this._tryNextToken()
      .then((connected) => {
        this._isConnecting = false;

        if (connected) {
          const token = this.secureToken;
          log.debug('got connected ; saving token', token);

          // Save the sucessful token
          this._saveToken(token);
          this._needsToken = false;

          // Emit the connected event
          log.debug('emitting "connected"');
          return this.emit('connected');
        }

        // If not connected, we need a new token
        log.debug('needs a token');
        this._needsToken = true;

        log.debug('emitting "connected"');
        return this.emit('disconnected');
      })
      .catch((error) => {
        this._isConnecting = false;

        log.debug('emitting "disconnected"');
        this.emit('disconnected');
        console.error('unhandled error in secureApi', error);
      });
  }

  _resetTokens () {
    this._tokens = this._tokens.map((token) => ({
      ...token,
      tried: false
    }));
  }

  _saveToken (token) {
    window.localStorage.setItem('sysuiToken', token);
  }

  /**
   * Returns a Promise that gets resolved with
   * a boolean: `true` if the node is up, `false`
   * otherwise (HEAD request to the Node)
   */
  isNodeUp () {
    const url = this._url.replace(/wss?/, 'http');
    return fetch(url, { method: 'HEAD' })
      .then(
        (r) => r.status === 200,
        () => false
      )
      .catch(() => false);
  }

  /**
   * Promise gets resolved when the node is up
   * and running (it might take some time before
   * the node is actually ready even when the client
   * is connected).
   *
   * We check that the `parity_enode` RPC calls
   * returns successfully
   */
  waitUntilNodeReady () {
    return this
      .parity.enode()
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
            this.waitUntilNodeReady().then(resolve).catch(reject);
          }, timeout);
        });
      });
  }

  /**
   * Try to connect to the Node with the next Token in
   * the list
   */
  _tryNextToken () {
    log.debug('trying next token');

    // Get the first not-tried token
    const nextTokenIndex = this._tokens.findIndex((t) => !t.tried);

    // If no more tokens to try, user has to enter a new one
    if (nextTokenIndex < 0) {
      return Promise.resolve(false);
    }

    const nextToken = this._tokens[nextTokenIndex];
    nextToken.tried = true;

    return this._connectWithToken(nextToken.value);
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

  _connectWithToken (_token) {
    // Sanitize the token first
    const token = this._sanitiseToken(_token);

    // Update the token in the transport layer
    this.transport.updateToken(token, false);
    log.debug('connecting with token', token);

    return this.transport.connect()
      .then(() => {
        log.debug('connected with', token);

        if (token === 'initial') {
          return this._generateAuthorizationToken();
        }

        return this.waitUntilNodeReady()
          .then(() => this._connectSuccess())
          .then(() => true);
      })
      .catch((error) => {
        // Log if it's not a close error (ie. wrong token)
        if (error && error.type !== 'close') {
          log.debug('did not connect ; error', error);
        }

        // Check if the Node is up
        return this.isNodeUp()
          .then((isNodeUp) => {
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

            log.debug('tried with a wrong token', token);
            return this._tryNextToken();
          });
      });
  }

  _connectSuccess () {
    // Retrieve the correct ports from the Node
    return Promise
      .all([
        this.parity.dappsPort(),
        this.parity.dappsInterface(),
        this.parity.signerPort()
      ])
      .then(([dappsPort, dappsInterface, signerPort]) => {
        this._dappsPort = dappsPort.toNumber();
        this._dappsInterface = dappsInterface;
        this._signerPort = signerPort.toNumber();
      });
  }

  _sanitiseToken (token) {
    return token.replace(/[^a-zA-Z0-9]/g, '');
  }

  updateToken (_token) {
    const token = this._sanitiseToken(_token);
    log.debug('updating token', token);

    // Update the tokens list: put the new one on first position
    this._tokens = [ { value: token, tried: false } ].concat(this._tokens);

    // Try to connect with the new token added
    return this.connect();
  }
}
