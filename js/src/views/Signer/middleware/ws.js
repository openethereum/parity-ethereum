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

import * as actions from '../actions/requests';

export default class LocalstorageMiddleware {
  constructor (ws, setToken) {
    this.setToken = setToken;
    this.ws = ws;
  }

  toMiddleware () {
    return store => next => action => {
      let delegate;
      switch (action.type) {
        case 'update token': delegate = this.onUpdateToken; break;
        case 'start confirmRequest': delegate = this.onConfirmStart; break;
        case 'start rejectRequest': delegate = this.onRejectStart; break;
        default:
          next(action);
          return;
      }

      if (!delegate) {
        return;
      }

      delegate(store, next, action);
    };
  }

  onUpdateToken = (store, next, action) => {
    this.setToken(action.payload);
    this.ws.init(action.payload);
    next(action);
  }

  onConfirmStart = (store, next, action) => {
    next(action);
    const { id, password } = action.payload;
    const method = 'personal_confirmRequest';

    this.send(method, [ id, {}, password ], (err, txHash) => {
      console.log('[WS MIDDLEWARE] confirm request cb:', err, txHash);
      if (err || !txHash) {
        store.dispatch(actions.errorConfirmRequest({ id, err: err ? err.message : 'Unable to confirm.' }));
        return;
      }

      store.dispatch(actions.successConfirmRequest({ id, txHash }));
      return;
    });
  }

  onRejectStart = (store, next, action) => {
    next(action);
    const id = action.payload;
    const method = 'personal_rejectRequest';

    this.send(method, [ id ], (err, res) => {
      console.log('[WS MIDDLEWARE] reject request cb:', err, res);
      if (err) {
        store.dispatch(actions.errorRejectRequest({ id, err: err.message }));
        return;
      }

      store.dispatch(actions.successRejectRequest({ id }));
    });
  }

  send (method, params, callback) {
    const payload = {
      jsonrpc: '2.0',
      method, params
    };
    this.ws.send(payload, callback);
  }

}
