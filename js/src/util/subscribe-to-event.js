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

import EventEmitter from 'eventemitter3';

const defaults = {
  from: 0, // TODO
  to: 'latest',
  timeout: null,
  filter: () => true
};

const subscribeToEvent = (contract, name, opt = {}) => {
  opt = Object.assign({}, defaults, opt);

  let subscription = null;
  let timeout = null;

  const unsubscribe = () => {
    if (subscription) {
      contract.unsubscribe(subscription);
      subscription = null;
    }
    if (timeout) {
      clearTimeout(timeout);
      timeout = null;
    }
  };

  if (typeof opt.timeout === 'number') {
    timeout = setTimeout(unsubscribe, opt.timeout);
  }

  const emitter = new EventEmitter();
  emitter.unsubscribe = unsubscribe;

  const callback = (err, logs) => {
    if (err) {
      return emitter.emit('error', err);
    }
    for (let log of logs) {
      if (opt.filter(log)) {
        emitter.emit('log', log);
      }
    }
  };

  contract.subscribe(name, {
    fromBlock: opt.from, toBlock: opt.to
  }, callback)
  .then((_subscription) => {
    subscription = _subscription;
  })
  .catch((err) => {
    emitter.emit('error', err);
  });

  return emitter;
};

export default subscribeToEvent;
