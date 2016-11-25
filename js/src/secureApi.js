// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import Api from './api';

const sysuiToken = window.localStorage.getItem('sysuiToken');

export default class SecureApi extends Api {
  constructor (url, nextToken) {
    super(new Api.Transport.Ws(url, sysuiToken));

    this._isConnecting = true;
    this._connectState = sysuiToken === 'initial' ? 1 : 0;
    this._needsToken = false;
    this._dappsPort = 8080;
    this._dappsInterface = null;
    this._signerPort = 8180;
    this._followConnectionTimeoutId = null;

    // Try tokens from localstorage, then from hash
    this._tokensToTry = [ sysuiToken, nextToken ].filter((t) => t && t.length);

    this._followConnection();
  }

  setToken = () => {
    window.localStorage.setItem('sysuiToken', this._transport.token);
    console.log('SecureApi:setToken', this._transport.token);
  }

  _checkNodeUp () {
    return fetch('/', { method: 'HEAD' })
      .then(
        (r) => r.status === 200,
        () => false
      )
      .catch(() => false);
  }

  _followConnection = () => {
    const nextTick = () => {
      if (this._followConnectionTimeoutId) {
        clearTimeout(this._followConnectionTimeoutId);
      }

      this._followConnectionTimeoutId = setTimeout(() => this._followConnection(), 250);
    };

    const setManual = () => {
      this._connectState = 100;
      this._needsToken = true;
      this._isConnecting = false;
    };

    const lastError = this._transport.lastError;
    const isConnected = this._transport.isConnected;

    switch (this._connectState) {
      // token = <passed via constructor>
      case 0:
        if (isConnected) {
          return this.connectSuccess();
        } else if (lastError) {
          return this
            ._checkNodeUp()
            .then((isNodeUp) => {
              const { timestamp } = lastError;

              if ((Date.now() - timestamp) > 250) {
                return nextTick();
              }

              const nextToken = this._tokensToTry[0] || 'initial';
              const nextState = nextToken !== 'initial' ? 0 : 1;

              // If previous token was wrong (error while node up), delete it
              if (isNodeUp) {
                this._tokensToTry = this._tokensToTry.slice(1);
              }

              if (nextToken !== this._transport.token) {
                this.updateToken(nextToken, nextState);
              }

              return nextTick();
            });
        }
        break;

      // token = 'initial'
      case 1:
        if (isConnected) {
          this.signer
            .generateAuthorizationToken()
            .then((token) => {
              this.updateToken(token, 2);
            })
            .catch((error) => {
              console.error('SecureApi:generateAuthorizationToken', error);
              setManual();
            });
          return;
        } else if (lastError) {
          return setManual();
        }
        break;

      // token = <personal_generateAuthorizationToken>
      case 2:
        if (isConnected) {
          return this.connectSuccess();
        } else if (lastError) {
          return setManual();
        }
        break;
    }

    nextTick();
  }

  connectSuccess () {
    this._isConnecting = false;
    this._needsToken = false;

    this.setToken();

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

    console.log('SecureApi:connectSuccess', this._transport.token);
  }

  updateToken (token, connectState = 0) {
    this._connectState = connectState;
    this._transport.updateToken(token.replace(/[^a-zA-Z0-9]/g, ''));
    this._followConnection();
    console.log('SecureApi:updateToken', this._transport.token, connectState);
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
