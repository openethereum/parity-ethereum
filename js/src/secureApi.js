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
  constructor (url) {
    super(new Api.Transport.Ws(url, sysuiToken));

    this._isConnecting = true;
    this._connectState = 0;
    this._needsToken = false;

    this._followConnection();
  }

  setToken = () => {
    window.localStorage.setItem('sysuiToken', this._transport.token);
  }

  _followConnection = () => {
    const nextTick = () => {
      setTimeout(this._followConnection, 250);
    };
    const setManual = () => {
      this._connectedState = 100;
      this._needsToken = true;
      this._isConnecting = false;
    };
    const lastError = this._transport.lastError;
    const isConnected = this._transport.isConnected;

    switch (this._connectState) {
      // token = <passed via constructor>
      case 0:
        if (isConnected) {
          this._isConnecting = false;
          return this.setToken();
        } else if (lastError) {
          this.updateToken('initial', 1);
        }
        break;

      // token = 'initial'
      case 1:
        if (isConnected) {
          this._connectState = 2;
          this.personal
            .generateAuthorizationToken()
            .then((token) => {
              this.updateToken(token, 2);
            })
            .catch((error) => {
              console.error('_followConnection', error);
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
          this._isConnecting = false;
          return this.setToken();
        } else if (lastError) {
          return setManual();
        }
        break;
    }

    nextTick();
  }

  updateToken (token, connectedState = 0) {
    this._connectState = connectedState;
    this._transport.updateToken(token.replace(/[^a-zA-Z0-9]/g, ''));
    this._followConnection();
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
