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

export default class Shh {
  constructor (transport) {
    this._transport = transport;
  }

  info () {
    return this._transport
      .execute('shh_info');
  }

  newKeyPair () {
    return this._transport
      .execute('shh_newKeyPair');
  }

  addPrivateKey (privKey) {
    return this._transport
      .execute('shh_addPrivateKey', privKey);
  }

  newSymKey () {
    return this._transport
      .execute('shh_newSymKey');
  }

  getPublicKey (identity) {
    return this._transport
      .execute('shh_getPublicKey', identity);
  }

  getPrivateKey (identity) {
    return this._transport
      .execute('shh_getPrivateKey', identity);
  }

  getSymKey (identity) {
    return this._transport
      .execute('shh_getSymKey', identity);
  }

  deleteKey (identity) {
    return this._transport
      .execute('shh_deleteKey', identity);
  }

  post (messageObj) {
    return this._transport
      .execute('shh_post', messageObj);
  }

  newMessageFilter (filterObj) {
    return this._transport
      .execute('shh_newMessageFilter', filterObj);
  }

  getFilterMessages (filterId) {
    return this._transport
      .execute('shh_getFilterMessages', filterId);
  }

  deleteMessageFilter (filterId) {
    return this._transport
      .execute('shh_deleteMessageFilter', filterId);
  }

  subscribe (filterObj, callback) {
    return this._transport
      .subscribe('shh', callback, filterObj);
  }

  unsubscribe (subscriptionId) {
    return this._transport
      .unsubscribe(subscriptionId);
  }
}
