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

const EventEmitter = require('eventemitter3');

const { toHex } = require('@parity/api/lib/util/format');
const { TransportError } = require('@parity/api/lib/transport');

const AWAITING = Symbol('awaiting');
const LOCKED = Symbol('locked');
const CONFIRMED = Symbol('confirmed');
const REJECTED = Symbol('rejected');

class Transactions extends EventEmitter {
  constructor () {
    super();

    this.reset();
  }

  // should only really be needed in the constructor and tests
  reset () {
    this._id = 1;
    this._states = {};
  }

  nextId () {
    return toHex(this._id++);
  }

  add (tx) {
    const id = this.nextId();

    this._states[id] = {
      status: AWAITING,
      transaction: tx
    };

    this.emit('update');

    return id;
  }

  get (id) {
    const state = this._states[id];

    if (!state || state.status !== AWAITING) {
      return null;
    }

    return state.transaction;
  }

  lock (id) {
    const state = this._states[id];

    if (!state || state.status !== AWAITING) {
      throw new Error('Trying to lock an invalid transaction');
    }

    state.status = LOCKED;

    this.emit('update');
  }

  unlock (id) {
    const state = this._states[id];

    if (!state || state.status !== LOCKED) {
      throw new Error('Trying to unlock an invalid transaction');
    }

    state.status = AWAITING;

    this.emit('update');
  }

  hash (id) {
    const state = this._states[id];

    if (!state) {
      return null;
    }

    switch (state.status) {
      case REJECTED:
        throw TransportError.requestRejected();
      case CONFIRMED:
        return state.hash;
      default:
        return null;
    }
  }

  confirm (id, hash) {
    const state = this._states[id];
    const status = state ? state.status : null;

    switch (status) {
      case AWAITING: break;
      case LOCKED: break;
      default: throw new Error('Trying to confirm an invalid transaction');
    }

    state.hash = hash;
    state.status = CONFIRMED;

    this.emit('update');
  }

  reject (id) {
    const state = this._states[id];

    if (!state) {
      return false;
    }

    state.status = REJECTED;

    this.emit('update');

    return true;
  }

  requestsToConfirm () {
    const result = [];

    Object.keys(this._states).forEach((id) => {
      const state = this._states[id];

      if (state.status === AWAITING) {
        result.push({
          id,
          origin: {
            signer: '0x0'
          },
          payload: {
            sendTransaction: state.transaction
          }
        });
      }
    });

    return result;
  }
}

module.exports = new Transactions();
