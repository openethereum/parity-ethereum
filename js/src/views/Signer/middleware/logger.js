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

import logger from '../utils/logger';

export default store => next => action => {
  if (store.getState().signer.logging) {
    const msg = [`[${now()}] action:`, `${action.type};`, 'payload: ', action.payload];
    // const logMethod = action.type.indexOf('error') > -1 ? 'error' : 'log';
    logger.log(...msg); // todo [adgo] - implement error logs
  }
  return next(action);
};

function now () {
  const date = new Date(Date.now());
  const seconds = date.getSeconds();
  const minutes = date.getMinutes();
  const hour = date.getHours();
  return `${hour}::${minutes}::${seconds}`;
}
