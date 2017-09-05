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

import EventEmitter from 'eventemitter3';

const defaults = {
  from: 0,
  to: 'latest',
  interval: 5000,
  filter: () => true
};

const subscribeToEvents = (contract, events, opt = {}) => {
  const { api } = contract;

  opt = Object.assign({}, defaults, opt);

  let filter = null;
  let interval = null;

  const unsubscribe = () => {
    if (filter) {
      filter
        .then((filterId) => {
          return api.eth.uninstallFilter(filterId);
        })
        .catch((err) => {
          emitter.emit('error', err);
        });
      filter = null;
    }
    if (interval) {
      clearInterval(interval);
      interval = null;
    }
  };

  const emitter = new EventEmitter();

  emitter.unsubscribe = unsubscribe;

  const fetcher = (method, filterId) => () => {
    api
      .eth[method](filterId)
      .then((logs) => {
        logs = contract.parseEventLogs(logs);

        for (let log of logs) {
          if (opt.filter(log)) {
            emitter.emit('log', log);
            emitter.emit(log.event, log);
          }
        }
      })
      .catch((err) => {
        emitter.emit('error', err);
      });
  };

  const signatures = events
    .filter((event) => contract.instance[event])
    .map((event) => contract.instance[event].signature);

  filter = api.eth
    .newFilter({
      fromBlock: opt.from,
      toBlock: opt.to,
      address: contract.address,
      topics: [signatures]
    })
    .then((filterId) => {
      fetcher('getFilterLogs', filterId)(); // fetch immediately

      const fetchChanges = fetcher('getFilterChanges', filterId);

      interval = setInterval(fetchChanges, opt.interval);

      return filterId;
    })
    .catch((err) => {
      emitter.emit('error', err);
      throw err; // reject Promise
    });

  return emitter;
};

export default subscribeToEvents;
