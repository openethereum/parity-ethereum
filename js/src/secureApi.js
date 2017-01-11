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
  constructor (url, nextToken) {
    const transport = new Api.Transport.Ws(url, sysuiToken, false);
    super(transport);

    this._url = url;
    this._isConnecting = true;
    this._needsToken = false;

    this._dappsPort = 8080;
    this._dappsInterface = null;
    this._signerPort = 8180;

    // Try tokens from localstorage, then from hash
    this._tokens = uniq([sysuiToken, nextToken, 'initial'])
      .filter((token) => token)
      .map((token) => ({ value: token, tried: false }));

    this.connect();
  }

  connect (token) {
    if (this._connectPromise) {
      return this
        ._connectPromise
        .then((connected) => {
          if (!connected && token) {
            return this._followConnection(token);
          }

          return connected;
        });
    }

    log.debug('trying to connect...');

    this._resetTokens();
    const promise = token
      ? this._followConnection(token)
      : this._tryNextToken();

    this._connectPromise = promise
      .then((connected) => {
        log.debug('got connected?', connected);

        this._connectPromise = null;
        return connected;
      });

    return this._connectPromise;
  }

  _resetTokens () {
    this._tokens = this._tokens.map((token) => ({
      ...token,
      tried: false
    }));
  }

  saveToken (token) {
    window.localStorage.setItem('sysuiToken', token);
    // DEBUG: console.log('SecureApi:saveToken', this._transport.token);
  }

  /**
   * Returns a Promise that gets resolved with
   * a boolean: `true` if the node is up, `false`
   * otherwise
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
          return false;
        }

        return new Promise((resolve, reject) => {
          window.setTimeout(() => {
            this.waitUntilNodeReady().then(resolve).catch(reject);
          }, 250);
        });
      });
  }

  _setManual () {
    this._needsToken = true;
    this._isConnecting = false;

    return false;
  }

  _tryNextToken () {
    log.debug('trying next token');

    const nextTokenIndex = this._tokens.findIndex((t) => !t.tried);

    if (nextTokenIndex < 0) {
      return this._setManual();
    }

    const nextToken = this._tokens[nextTokenIndex];
    nextToken.tried = true;

    return this._followConnection(nextToken.value);
  }

  _followConnection (_token) {
    const token = this._sanitiseToken(_token);
    this.transport.updateToken(token, false);
    log.debug('connecting with token', token);

    return this
      .transport
      .connect()
      .then(() => {
        log.debug('connected with', token);

        if (token === 'initial') {
          return this.signer
            .generateAuthorizationToken()
            .then((token) => {
              return this._followConnection(token);
            })
            .catch(() => {
              return false;
            });
        }

        return this.waitUntilNodeReady().then(() => {
          return this.connectSuccess(token).then(() => true, () => true);
        });
      })
      .catch((error) => {
        if (error && error.type !== 'close') {
          log.debug('did not connect ; error', e);
        }

        return this
          .isNodeUp()
          .then((isNodeUp) => {
            log.debug('did not connect with', token, '; is node up?', isNodeUp ? 'yes' : 'no');

            // Try again in a few...
            if (!isNodeUp) {
              this._isConnecting = false;
              const timeout = this.transport.retryTimeout;

              return new Promise((resolve, reject) => {
                window.setTimeout(() => {
                  this._followConnection(token).then(resolve).catch(reject);
                }, timeout);
              });
            }

            return this._tryNextToken();
          });
      });
  }

  connectSuccess (token) {
    this._isConnecting = false;
    this._needsToken = false;

    this.saveToken(token);
    log.debug('got connected ; saving token', token);

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

    // DEBUG: console.log('SecureApi:connectSuccess', this._transport.token);
  }

  _sanitiseToken (token) {
    return token.replace(/[^a-zA-Z0-9]/g, '');
  }

  updateToken (_token) {
    const token = this._sanitiseToken(_token);
    log.debug('updating token', token);

    // Update the tokens list
    this._tokens = this._tokens.concat([ { value: token, tried: false } ]);

    return this.connect(token);
    // DEBUG: console.log('SecureApi:updateToken', this._transport.token, connectState);
  }

  get dappsPort () {
    return this._dappsPort;
  }

  get dappsUrl () {
    let hostname;

    if (window.location.hostname === 'home.parity') {
      hostname = 'dapps.parity';
    } else if (!this._dappsInterface || this._dappsInterface === '0.0.0.0') {
      hostname = window.location.hostname;
    } else {
      hostname = this._dappsInterface;
    }

    return `http://${hostname}:${this._dappsPort}`;
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
}
