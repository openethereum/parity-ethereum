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

import { Web3Base } from '../provider/web3-base';

export default class WebInteractions extends Web3Base {

  toMiddleware () {
    return store => next => action => {
      let delegate;
      if (action.type.indexOf('modify ') > -1) {
        delegate = ::this.onModify;
      } else {
        next(action);
        return;
      }

      if (!delegate) {
        return;
      }

      delegate(store, next, action);
    };
  }

  onModify (store, next, action) {
    this.ethcoreWeb3[this.getMethod(action.type)](action.payload);
    action.type = action.type.replace('modify ', 'update ');
    return next(action);
  }

  getMethod (actionType) {
    let method = actionType.split('modify ')[1];
    return 'set' + method[0].toUpperCase() + method.slice(1);
  }
}
