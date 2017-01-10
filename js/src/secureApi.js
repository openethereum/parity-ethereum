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

    this._tryNextToken();
  }

  saveToken () {
    window.localStorage.setItem('sysuiToken', this._transport.token);
    // DEBUG: console.log('SecureApi:saveToken', this._transport.token);
  }

  /**
   * Returns a Promise that gets resolved with
   * a boolean: `true` if the node is up, `false`
   * otherwise
   */
  _checkNodeUp () {
    const url = this._url.replace(/wss?/, 'http');
    return fetch(url, { method: 'HEAD' })
      .then(
        (r) => r.status === 200,
        () => false
      )
      .catch(() => false);
  }

  _setManual () {
    this._needsToken = true;
    this._isConnecting = false;
  }

  _tryNextToken () {
    const nextTokenIndex = this._tokens.findIndex((t) => !t.tried);

    if (nextTokenIndex < 0) {
      return this._setManual();
    }

    const nextToken = this._tokens[nextTokenIndex];
    nextToken.tried = true;

    this.updateToken(nextToken.value);
  }

  _followConnection = () => {
    const token = this.transport.token;

    return this
      .transport
      .connect()
      .then(() => {
        if (token === 'initial') {
          return this.signer
            .generateAuthorizationToken()
            .then((token) => {
              return this.updateToken(token);
            })
            .catch((e) => console.error(e));
        }

        this.connectSuccess();
        return true;
      })
      .catch((e) => {
        this
          ._checkNodeUp()
          .then((isNodeUp) => {
            // Try again in a few...
            if (!isNodeUp) {
              this._isConnecting = false;
              const timeout = this.transport.retryTimeout;

              window.setTimeout(() => {
                this._followConnection();
              }, timeout);

              return;
            }

            this._tryNextToken();
            return false;
          });
      });
  }

  connectSuccess () {
    this._isConnecting = false;
    this._needsToken = false;

    this.saveToken();

    Promise
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

  updateToken (token) {
    this._transport.updateToken(token.replace(/[^a-zA-Z0-9]/g, ''), false);
    return this._followConnection();
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
